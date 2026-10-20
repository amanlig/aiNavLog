# Synthetic Victron ciphertext fixtures

These are generated test data, not captures or device credentials. Product ID
`0x1234` is arbitrary. The public AES-128 key is
`000102030405060708090a0b0c0d0e0f`; nonce bytes are `34 12`.

Encrypt the hexadecimal plaintext below (converted to bytes) with OpenSSL:

```sh
openssl enc -aes-128-ctr -K 000102030405060708090a0b0c0d0e0f \
  -iv 34120000000000000000000000000000 -nopad
```

Prepend `10 02 34 12 TYPE 34 12 00` to its binary output. TYPE is `04` for
DC/DC and `0f` for XS. Committed files are consumed directly by tests; OpenSSL
is not required to run the test suite.

| File | Type | Plaintext (hex) |
| --- | --- | --- |
| orion_tr.bin | 04 | `03002805a00581000080` |
| orion_tr_na.bin | 04 | `ffffffffff7fffffffff` |
| orion_tr_negative.bin | 04 | `fefe000083ff00000000` |
| orion_xs.bin | 0f | `0300a00585ff2805960081000080` |
| orion_xs_na.bin | 0f | `ffffff7fff7fffffffffffffffff` |
| orion_tr_extended.bin | 04 | `03002805a00581000080000102030405060708090a0b0c0d0e0f1011121314151617` |

The normal DC/DC fixture has state 3, error 0, 13.2 V input, 14.4 V output
and off-reason bits `0x80000081`. The XS fixture adds -12.3 A output current
and 15 A input current. NA fixtures exercise published unavailable sentinels.

BMV fixtures use header `10 02 34 12 02 34 12 00` and the same public key/IV.
`bmv_na.bin` plaintext: `ffffff7f0000ffffffffffffffffffff` (measurements NA,
auxiliary disabled). `bmv_starter.bin` is packed little-endian: TTG 120, battery
1276 centivolts, alarm 0, auxiliary 1280 centivolts, auxiliary selector 0,
signed22 current -5250 mA, consumed 450 deci-Ah, SOC 775 deci-percent, reserved
bits set. Both were encrypted independently with OpenSSL. Charge/discharge
fixtures live under simulation/bmv712 and are documented there.
