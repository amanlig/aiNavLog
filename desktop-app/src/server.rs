use ainavlog_local_storage::ObservationStore;
use serde_json::json;
use std::{
    io::{Read, Write},
    net::TcpStream,
    path::Path,
    time::Duration,
};
include!(concat!(env!("OUT_DIR"), "/assets.rs"));
const ID: &str = "aiNavLog desktop 1";
pub fn new_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    #[cfg(windows)]
    {
        #[link(name = "bcrypt")]
        unsafe extern "system" {
            fn BCryptGenRandom(
                algorithm: *mut std::ffi::c_void,
                buffer: *mut u8,
                length: u32,
                flags: u32,
            ) -> i32;
        }
        if unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                bytes.as_mut_ptr(),
                bytes.len() as u32,
                2,
            )
        } != 0
        {
            return Err("Could not initialize local session".into());
        }
    }
    #[cfg(not(windows))]
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|v| format!("{v:02x}")).collect())
}
pub fn existing_host(port: u16) -> bool {
    let result = (|| -> std::io::Result<bool> {
        let mut s = TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_secs(1),
        )?;
        s.set_read_timeout(Some(Duration::from_secs(1)))?;
        write!(
            s,
            "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
        )?;
        let mut data = String::new();
        s.take(4096).read_to_string(&mut data)?;
        Ok(data.ends_with(ID))
    })();
    result.unwrap_or(false)
}
fn respond(stream: &mut TcpStream, status: &str, mime: &str, body: &[u8]) -> std::io::Result<()> {
    write!(stream,"HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nContent-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'\r\nConnection: close\r\n\r\n",body.len())?;
    stream.write_all(body)
}
pub fn handle(
    mut stream: TcpStream,
    database: &Path,
    address: &str,
    token: &str,
) -> std::io::Result<()> {
    // Windows accepts may inherit the nonblocking listener mode.
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut data = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        if let Some(i) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
        if data.len() > 16384 {
            return respond(
                &mut stream,
                "431 Request Header Fields Too Large",
                "text/plain",
                b"Request too large",
            );
        }
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Ok(());
        }
        data.extend_from_slice(&chunk[..n]);
    };
    let headers = String::from_utf8_lossy(&data[..header_end]).into_owned();
    let mut first = headers.lines().next().unwrap_or("").split_whitespace();
    let method = first.next().unwrap_or("");
    let target = first.next().unwrap_or("");
    let header = |name: &str| {
        headers
            .lines()
            .skip(1)
            .filter_map(|l| l.split_once(':'))
            .filter(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.trim())
            .collect::<Vec<_>>()
    };
    if header("Host") != [address] {
        return respond(&mut stream, "403 Forbidden", "text/plain", b"Invalid host");
    }
    let origin = format!("http://{address}");
    if method == "POST" && target == "/maintenance" {
        if header("Origin") != [origin.as_str()]
            || header("X-AiNavLog-Token") != [token]
            || header("Content-Type") != ["application/json"]
            || !header("Transfer-Encoding").is_empty()
        {
            return respond(
                &mut stream,
                "403 Forbidden",
                "application/json",
                br#"{"error":"Invalid local session"}"#,
            );
        }
        let lengths = header("Content-Length");
        let length = if lengths.len() == 1 {
            lengths[0].parse::<usize>().ok().filter(|v| *v <= 32768)
        } else {
            None
        };
        let Some(length) = length else {
            return respond(
                &mut stream,
                "413 Payload Too Large",
                "application/json",
                b"{}",
            );
        };
        let expectations = header("Expect");
        if !expectations.is_empty() {
            if expectations.len() != 1 || !expectations[0].eq_ignore_ascii_case("100-continue") {
                return respond(
                    &mut stream,
                    "417 Expectation Failed",
                    "application/json",
                    b"{}",
                );
            }
            stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n")?;
        }
        while data.len() - header_end < length {
            let n = stream.read(&mut chunk)?;
            if n == 0 {
                return respond(&mut stream, "400 Bad Request", "application/json", b"{}");
            }
            data.extend_from_slice(&chunk[..n]);
        }
        let Ok(request) = std::str::from_utf8(&data[header_end..header_end + length]) else {
            return respond(&mut stream, "400 Bad Request", "application/json", b"{}");
        };
        let result = ainavlog_native_bridge::maintenance::execute(database, request);
        return respond(&mut stream, "200 OK", "application/json", result.as_bytes());
    }
    if method != "GET" {
        return respond(
            &mut stream,
            "405 Method Not Allowed",
            "text/plain",
            b"GET required",
        );
    }
    if target == "/health" {
        return respond(&mut stream, "200 OK", "text/plain", ID.as_bytes());
    }
    if target == "/electrical" {
        return respond(&mut stream, "200 OK", "application/json", b"null");
    }
    if let Some(query) = target.strip_prefix("/electrical/history?") {
        let range = (|| {
            let mut start = None;
            let mut end = None;
            for pair in query.split('&') {
                let (k, v) = pair.split_once('=')?;
                let v = v.parse::<i64>().ok()?;
                match k {
                    "start" if start.is_none() => start = Some(v),
                    "end" if end.is_none() => end = Some(v),
                    _ => return None,
                }
            }
            Some((start?, end?))
        })();
        if let Some((start, end)) = range {
            if let Ok(points) =
                ObservationStore::read_bmv_history(database, "windows-bmv-simulator", start, end)
            {
                let rows = points
                    .iter()
                    .map(|p| {
                        let values = [
                            p.house_voltage,
                            p.house_current,
                            p.house_charge,
                            p.starter_voltage,
                        ];
                        let mask = values
                            .iter()
                            .enumerate()
                            .fold(0, |m, (i, v)| m | if v.is_some() { 1 << i } else { 0 });
                        json!([
                            p.received_at_ms,
                            mask,
                            values[0].unwrap_or(0.),
                            values[1].unwrap_or(0.),
                            values[2].unwrap_or(0.),
                            values[3].unwrap_or(0.),
                            p.segment
                        ])
                    })
                    .collect::<Vec<_>>();
                return respond(
                    &mut stream,
                    "200 OK",
                    "application/json",
                    json!({"source":"simulated","points":rows})
                        .to_string()
                        .as_bytes(),
                );
            }
        }
        return respond(
            &mut stream,
            "400 Bad Request",
            "application/json",
            br#"{"error":"Invalid history query"}"#,
        );
    }
    let route = target.split('?').next().unwrap_or("");
    let asset = ASSETS
        .iter()
        .find(|(name, _, _)| *name == route)
        .or_else(|| {
            if matches!(
                route,
                "/" | "/elec"
                    | "/elec/history"
                    | "/elec/maintenance"
                    | "/mech"
                    | "/mech/maintenance"
                    | "/rout"
            ) {
                ASSETS.iter().find(|(name, _, _)| *name == "/index.html")
            } else {
                None
            }
        });
    let Some((_, mime, body)) = asset else {
        return respond(&mut stream, "404 Not Found", "text/plain", b"Not found");
    };
    if mime.starts_with("text/html") {
        let config = json!({"endpoint":format!("{origin}/electrical"),"token":token});
        let html = String::from_utf8_lossy(body).replacen(
            "<head>",
            &format!("<head><script>globalThis.__AINAVLOG_DESKTOP__={config};</script>"),
            1,
        );
        return respond(&mut stream, "200 OK", mime, html.as_bytes());
    }
    respond(&mut stream, "200 OK", mime, body)
}
