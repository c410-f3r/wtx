//! X.509 chain validation

extern crate wtx;

use core::slice;
use wtx::{
  asn1::parse_der_from_pem_range,
  codec::{Decode as _, DecodeWrapper},
  collection::Vector,
  misc::Pem,
  x509::{Certificate, CvEndEntity, CvIntermediate, CvPolicy, CvTrustAnchor},
};

const END_ENTITY: &[u8] = b"-----BEGIN CERTIFICATE-----
MIIFxDCCBWmgAwIBAgIQaD3YAMfWDUsaC+cNmW6poDAKBggqhkjOPQQDAjBRMQsw
CQYDVQQGEwJVUzETMBEGA1UEChMKQXBwbGUgSW5jLjEtMCsGA1UEAxMkQXBwbGUg
UHVibGljIEVWIFNlcnZlciBFQ0MgQ0EgMSAtIEcxMB4XDTI2MDIyNjE4MDcxNloX
DTI2MDUyNzE5MDk0OVowgcMxHTAbBgNVBA8MFFByaXZhdGUgT3JnYW5pemF0aW9u
MRMwEQYLKwYBBAGCNzwCAQMTAlVTMRswGQYLKwYBBAGCNzwCAQIMCkNhbGlmb3Ju
aWExETAPBgNVBAUTCEMwODA2NTkyMQswCQYDVQQGEwJVUzETMBEGA1UECAwKQ2Fs
aWZvcm5pYTESMBAGA1UEBwwJQ3VwZXJ0aW5vMRMwEQYDVQQKDApBcHBsZSBJbmMu
MRIwEAYDVQQDDAlhcHBsZS5jb20wWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAATF
8Ac3KZPGRdrd5PtJta681gjvUlVA7qR+iWw3/JB+PI48vOoYzTw86Z/W6hrlmcoQ
S6pRj03OEmmcWXkTB1e4o4IDrjCCA6owDAYDVR0TAQH/BAIwADAfBgNVHSMEGDAW
gBTghUh9E6bTEBmfXMtreCSS+K4brjB6BggrBgEFBQcBAQRuMGwwMgYIKwYBBQUH
MAKGJmh0dHA6Ly9jZXJ0cy5hcHBsZS5jb20vYXBldnNlY2MxZzEuZGVyMDYGCCsG
AQUFBzABhipodHRwOi8vb2NzcC5hcHBsZS5jb20vb2NzcDAzLWFwZXZzZWNjMWcx
MDEwFAYDVR0RBA0wC4IJYXBwbGUuY29tMGAGA1UdIARZMFcwSAYFZ4EMAQEwPzA9
BggrBgEFBQcCARYxaHR0cHM6Ly93d3cuYXBwbGUuY29tL2NlcnRpZmljYXRlYXV0
aG9yaXR5L3B1YmxpYzALBglghkgBhv1sAgEwEwYDVR0lBAwwCgYIKwYBBQUHAwEw
NQYDVR0fBC4wLDAqoCigJoYkaHR0cDovL2NybC5hcHBsZS5jb20vYXBldnNlY2Mx
ZzEuY3JsMB0GA1UdDgQWBBQjqdcO+ifokErbZnTjOrNR1SgNiTAOBgNVHQ8BAf8E
BAMCB4AwDwYJKoZIhvdjZAZWBAIFADCCAfcGCisGAQQB1nkCBAIEggHnBIIB4wHh
AHYAlpdkv1VYl633Q4doNwhCd+nwOtX2pPM2bkakPw/KqcYAAAGcmythYgAABAMA
RzBFAiEA1PhX+2cAc4EW0geuD07k34wvWCOzfzcDbkN4IACLxCECIFm2mf+KrYxz
POfCTAy19oRfnya5HVqBytwL+MPUgFlxAHcAZBHEbKQS7KeJHKICLgC8q08oB9Qe
NSer6v7VA8l9zfAAAAGcmythHwAABAMASDBGAiEAyZ8Fg+8QrV0CaXhIMZexYTdM
D2KOaUJf7SwF3DvDWR0CIQCyPDXDHMgaAC8oBPHSRBIzbE9M2LkqpNcpOq/yThzZ
KQB2ABaDLavwqSUPD/A6pUX/yL/II9CHS/YEKSf45x8zE/X6AAABnJsrYUMAAAQD
AEcwRQIgJbU/WVxUUOdhNCy/UpEjbqYFI1NXQrIyuvqIJNTWMGUCIQDzCVTxeChI
em7lS5ISRhcbbwbznmKNIaFh2f3ITWDGRgB2AMs49xWJfIShRF9bwd37yW7ymlnN
RwppBYWwyxTDFFjnAAABnJsrYToAAAQDAEcwRQIgP3cimH+o3cyCPT9yPyqS39H+
ec0utyDDYBSr45J4BPMCIQD4KNwoJWADwMAWuSfjAL9sxTH2hHdAA0gjnKrRPYT7
GjAKBggqhkjOPQQDAgNJADBGAiEAtsvO+N4V5m9WAsC12+qaJS5WFhTxoVmy/b34
v4Y96TACIQCCLj3iOuotRmOrHyF6XXT6/XD5Lj4Yjd5pbOFmNLQ8vg==
-----END CERTIFICATE-----";

const INTERMEDIATE: &[u8] = b"-----BEGIN CERTIFICATE-----
MIIDsjCCAzigAwIBAgIQDKuq0c7E6XzCZliB0CE49zAKBggqhkjOPQQDAzBhMQsw
CQYDVQQGEwJVUzEVMBMGA1UEChMMRGlnaUNlcnQgSW5jMRkwFwYDVQQLExB3d3cu
ZGlnaWNlcnQuY29tMSAwHgYDVQQDExdEaWdpQ2VydCBHbG9iYWwgUm9vdCBHMzAe
Fw0yMDA0MjkxMjM0NTJaFw0zMDA0MTAyMzU5NTlaMFExCzAJBgNVBAYTAlVTMRMw
EQYDVQQKEwpBcHBsZSBJbmMuMS0wKwYDVQQDEyRBcHBsZSBQdWJsaWMgRVYgU2Vy
dmVyIEVDQyBDQSAxIC0gRzEwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAQp+OFa
uYdEBJj/FpCG+eDhQmVfhv0DGPzGz40TW8BeWxipYTOa4FLieAYoU+3t2tg9FZKt
A4BDTO43YprLZm6zo4IB4DCCAdwwHQYDVR0OBBYEFOCFSH0TptMQGZ9cy2t4JJL4
rhuuMB8GA1UdIwQYMBaAFLPbSKT5ocXYrjZBzBFjaWIpvEvGMA4GA1UdDwEB/wQE
AwIBhjAdBgNVHSUEFjAUBggrBgEFBQcDAQYIKwYBBQUHAwIwEgYDVR0TAQH/BAgw
BgEB/wIBADA0BggrBgEFBQcBAQQoMCYwJAYIKwYBBQUHMAGGGGh0dHA6Ly9vY3Nw
LmRpZ2ljZXJ0LmNvbTBCBgNVHR8EOzA5MDegNaAzhjFodHRwOi8vY3JsMy5kaWdp
Y2VydC5jb20vRGlnaUNlcnRHbG9iYWxSb290RzMuY3JsMIHcBgNVHSAEgdQwgdEw
gcUGCWCGSAGG/WwCATCBtzAoBggrBgEFBQcCARYcaHR0cHM6Ly93d3cuZGlnaWNl
cnQuY29tL0NQUzCBigYIKwYBBQUHAgIwfgx8QW55IHVzZSBvZiB0aGlzIENlcnRp
ZmljYXRlIGNvbnN0aXR1dGVzIGFjY2VwdGFuY2Ugb2YgdGhlIFJlbHlpbmcgUGFy
dHkgQWdyZWVtZW50IGxvY2F0ZWQgYXQgaHR0cHM6Ly93d3cuZGlnaWNlcnQuY29t
L3JwYS11YTAHBgVngQwBATAKBggqhkjOPQQDAwNoADBlAjEAyHLAT/4iBuxi4/NH
hZde4PZO8CnG2/A3oGO0Nsjpoe2SV94Hr+JpYHrBzT8hyeKSAjBnRXyRac9sM8KN
Fdg3+7LWIiW9sUjtJC6kGmRyGm6vV4oAhEDd9jdk4q+7b5zlid4=
-----END CERTIFICATE-----";

const TRUST_ANCHOR: &[u8] = b"-----BEGIN CERTIFICATE-----
MIICPzCCAcWgAwIBAgIQBVVWvPJepDU1w6QP1atFcjAKBggqhkjOPQQDAzBhMQsw
CQYDVQQGEwJVUzEVMBMGA1UEChMMRGlnaUNlcnQgSW5jMRkwFwYDVQQLExB3d3cu
ZGlnaWNlcnQuY29tMSAwHgYDVQQDExdEaWdpQ2VydCBHbG9iYWwgUm9vdCBHMzAe
Fw0xMzA4MDExMjAwMDBaFw0zODAxMTUxMjAwMDBaMGExCzAJBgNVBAYTAlVTMRUw
EwYDVQQKEwxEaWdpQ2VydCBJbmMxGTAXBgNVBAsTEHd3dy5kaWdpY2VydC5jb20x
IDAeBgNVBAMTF0RpZ2lDZXJ0IEdsb2JhbCBSb290IEczMHYwEAYHKoZIzj0CAQYF
K4EEACIDYgAE3afZu4q4C/sLfyHS8L6+c/MzXRq8NOrexpu80JX28MzQC7phW1FG
fp4tn+6OYwwX7Adw9c+ELkCDnOg/QW07rdOkFFk2eJ0DQ+4QE2xy3q6Ip6FrtUPO
Z9wj/wMco+I+o0IwQDAPBgNVHRMBAf8EBTADAQH/MA4GA1UdDwEB/wQEAwIBhjAd
BgNVHQ4EFgQUs9tIpPmhxdiuNkHMEWNpYim8S8YwCgYIKoZIzj0EAwMDaAAwZQIx
AK288mw/EkrRLTnDCgmXc/SINoyIJ7vmiI1Qhadj+Z4y3maTD/HMsQmP3Wyr+mt/
oAIwOWZbwmSNuJ5Q3KjVSaLtx9zRSX8XAbjIho9OjIgrqJqpisXRAL34VOKa5Vt8
sycX
-----END CERTIFICATE-----";

// Feel free to choose between `single_buffer` and `multiple_buffers`. They do the same thing in
// different ways.
fn main() -> wtx::Result<()> {
  single_buffer()?;
  multiple_buffers()?;
  Ok(())
}

fn multiple_buffers() -> wtx::Result<()> {
  validate_chain(
    &CvEndEntity::try_from(Certificate::from_pem(&mut Vector::new(), END_ENTITY)?)?,
    &CvIntermediate::try_from(Certificate::from_pem(&mut Vector::new(), INTERMEDIATE)?)?,
    &CvTrustAnchor::try_from(Certificate::from_pem(&mut Vector::new(), TRUST_ANCHOR)?)?,
  )
}

fn single_buffer() -> wtx::Result<()> {
  let mut buffer = Vector::new();

  let end_entity_range = Pem::decode(&mut DecodeWrapper::new(END_ENTITY, &mut buffer))?;
  let intermediate_range = Pem::decode(&mut DecodeWrapper::new(INTERMEDIATE, &mut buffer))?;
  let trust_anchor_range = Pem::decode(&mut DecodeWrapper::new(TRUST_ANCHOR, &mut buffer))?;

  let end_entity_cert: Certificate<'_> = parse_der_from_pem_range(&buffer, &end_entity_range)?;
  let intermediate_cert: Certificate<'_> = parse_der_from_pem_range(&buffer, &intermediate_range)?;
  let trust_anchor_cert: Certificate<'_> = parse_der_from_pem_range(&buffer, &trust_anchor_range)?;

  validate_chain(
    &CvEndEntity::try_from(end_entity_cert)?,
    &CvIntermediate::try_from(intermediate_cert)?,
    &CvTrustAnchor::try_from(trust_anchor_cert)?,
  )
}

fn validate_chain(
  end_entity: &CvEndEntity<'_, '_>,
  intermediate: &CvIntermediate<'_, '_>,
  trust_anchor: &CvTrustAnchor<'_, '_>,
) -> wtx::Result<()> {
  let cvp = CvPolicy::from_crls(&[])?;
  let verified_path = end_entity.validate_chain(
    slice::from_ref(intermediate),
    &cvp,
    slice::from_ref(trust_anchor),
  )?;
  assert_eq!(verified_path.end_entity(), end_entity);
  assert_eq!(verified_path.intermediates(), &[intermediate]);
  assert_eq!(verified_path.trust_anchor(), trust_anchor);
  Ok(())
}
