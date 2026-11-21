// Liveness only: no OpenAI request, credentials or provider spend.
try {
  const response=await fetch(`http://127.0.0.1:${process.env.PORT||8788}/healthz`,{signal:AbortSignal.timeout(3500)});
  if(!response.ok||(await response.json()).status!=='ok')process.exitCode=1;
}catch{process.exitCode=1;}
