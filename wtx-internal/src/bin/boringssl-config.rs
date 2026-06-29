// Creates the configuration file used by the BoringSSL's testsuite.

use std::fs;
use wtx::collections::{HashMap, Vector};

fn main() {
  let config = Config {
    disabled_tests: disabled_tests(),
    error_map: error_map(),
    half_rtt_tickets: 0,
    test_error_map: test_error_map(),
    test_local_error_map: test_local_error_map(),
  };
  let data = serde_json::to_vec_pretty(&config).unwrap();
  fs::write("boringssl-config.json", data).unwrap();
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct Config {
  disabled_tests: HashMap<&'static str, &'static str>,
  error_map: HashMap<&'static str, Vector<&'static str>>,
  test_error_map: HashMap<&'static str, &'static str>,
  test_local_error_map: HashMap<&'static str, &'static str>,
  #[serde(rename = "HalfRTTTickets")]
  half_rtt_tickets: u64,
}

fn disabled_tests() -> HashMap<&'static str, &'static str> {
  [
    ("*-ECDSA_SHA1-*", "UNSUPPORTED"),
    ("*-HintMismatch-*", "UNSUPPORTED"),
    ("*-P-224-*", "UNSUPPORTED"),
    ("*-QUIC", "UNSUPPORTED"),
    ("*-QUIC-*", "UNSUPPORTED"),
    ("*-RSA_PKCS1_SHA256_LEGACY-*", "UNSUPPORTED"),
    ("*-RSA_WITH_3DES_EDE_CBC_SHA-*", "UNSUPPORTED"),
    ("*-RSA_WITH_AES_128_CBC_SHA-*", "UNSUPPORTED"),
    ("*-RSA_WITH_AES_128_GCM_SHA256-*", "UNSUPPORTED"),
    ("*-RSA_WITH_AES_256_CBC_SHA-*", "UNSUPPORTED"),
    ("*-RSA_WITH_AES_256_GCM_SHA384-*", "UNSUPPORTED"),
    ("*-Sign-RSA_PKCS1_SHA1-*", "UNSUPPORTED"),
    ("*-SignDefault-RSA_PKCS1_SHA1-*", "UNSUPPORTED"),
    ("*-TLS1", "UNSUPPORTED"),
    ("*-TLS11", "UNSUPPORTED"),
    ("*-TLS12", "UNSUPPORTED"),
    ("*-VerifyDefault-RSA_PKCS1_SHA1-*", "UNSUPPORTED"),
    ("*Auth-SHA1-Fallback*", "UNSUPPORTED"),
    ("*CBCPadding*", "UNSUPPORTED"),
    ("*DTLS*", "UNSUPPORTED"),
    ("*EarlyKeyingMaterial-Client-*", "UNSUPPORTED"),
    ("*Kyber*", "UNSUPPORTED"),
    ("*SSL3*", "UNSUPPORTED"),
    ("*TLS1-*", "UNSUPPORTED"),
    ("*TLS11-*", "UNSUPPORTED"),
    ("*TLS12-*", "UNSUPPORTED"),
    ("*TLS12-*", "UNSUPPORTED"),
    ("*_P224_*", "UNSUPPORTED"),
    ("*_WITH_AES_128_CBC_*", "UNSUPPORTED"),
    ("*_WITH_AES_256_CBC_*", "UNSUPPORTED"),
    ("ALPN*SelectEmpty-*", "UNSUPPORTED"),
    ("ALPS-*", "UNSUPPORTED"),
    ("BadRSAClientKeyExchange-*", "UNSUPPORTED"),
    ("Basic-Server-RSA-*", "UNSUPPORTED"),
    ("CBCRecordSplitting*", "UNSUPPORTED"),
    ("CertificateCipherMismatch-PSS", "UNSUPPORTED"),
    ("CertificateSelection-*ClientCertificateTypes-*", "UNSUPPORTED"),
    ("CertificateSelection-*TrustAnchorIDs-*", "UNSUPPORTED"),
    ("CertificateSelection-Client-SignatureAlgorithmECDSACurve-TLS-TLS12", "UNSUPPORTED"),
    ("CertificateSelection-Server-*", "UNSUPPORTED"),
    ("CheckClientCertificateTypes", "UNSUPPORTED"),
    ("CheckECDSACurve-TLS12", "UNSUPPORTED"),
    ("CheckLeafCurve", "UNSUPPORTED"),
    ("CheckRecordVersion-*", "UNSUPPORTED"),
    ("Client-RejectJDK11DowngradeRandom", "UNSUPPORTED"),
    ("Client-VerifyDefault-ECDSA_P521_SHA512-TLS12", "UNSUPPORTED"),
    ("Client-VerifyDefault-ECDSA_P521_SHA512-TLS13", "UNSUPPORTED"),
    ("Client-VerifyDefault-Ed25519-TLS12", "UNSUPPORTED"),
    ("Client-VerifyDefault-Ed25519-TLS13", "UNSUPPORTED"),
    ("ClientHelloPadding", "UNSUPPORTED"),
    ("ConflictingVersionNegotiation", "UNSUPPORTED"),
    ("CurveTest-*-P-521-*", "UNSUPPORTED"),
    ("DelegatedCredentials-*", "UNSUPPORTED"),
    ("DisableEverything", "UNSUPPORTED"),
    ("DuplicateCertCompressionExt*-TLS12", "UNSUPPORTED"),
    ("ECDSAKeyUsage-*", "UNSUPPORTED"),
    ("EarlyData-*ALPN*-*", "UNSUPPORTED"),
    ("Ed25519DefaultDisable-NoAccept", "UNSUPPORTED"),
    ("Ed25519DefaultDisable-NoAdvertise", "UNSUPPORTED"),
    ("EmptyExtensions-ClientHello-TLS12", "UNSUPPORTED"),
    ("ExtendedMasterSecret-Renego-*", "UNSUPPORTED"),
    ("ExtraClientEncryptedExtension-*", "UNSUPPORTED"),
    ("FallbackSCSV*", "UNSUPPORTED"),
    ("GREASE-*", "UNSUPPORTED"),
    ("IgnoreExtensionsOnIntermediates-TLS13", "UNSUPPORTED"),
    ("LargeMessage-Reject", "UNSUPPORTED"),
    ("*MLKEM*", "UNSUPPORTED"),
    ("MTU*", "UNSUPPORTED"),
    ("NPN-*", "UNSUPPORTED"),
    ("NoCommonCurves", "UNSUPPORTED"),
    ("NoCommonSignatureAlgorithms-TLS12-Fallback", "UNSUPPORTED"),
    ("OmitExtensions-ClientHello-TLS12", "UNSUPPORTED"),
    ("PAKE-*", "UNSUPPORTED"),
    ("Peek-*", "UNSUPPORTED"),
    ("QUIC-*", "UNSUPPORTED"),
    ("QUICCompatibilityMode", "UNSUPPORTED"),
    ("QUICTransportParams-*", "UNSUPPORTED"),
    ("RSA-PSS-Large", "UNSUPPORTED"),
    ("RSAEphemeralKey", "UNSUPPORTED"),
    ("RSAKeyUsage-*", "UNSUPPORTED"),
    ("Renegotiate-Client-*", "UNSUPPORTED"),
    ("Renegotiate-ForbidAfterHandshake", "UNSUPPORTED"),
    ("Renegotiate-Server-*", "UNSUPPORTED"),
    ("RequireAnyClientCertificate-TLS12", "UNSUPPORTED"),
    ("Resume-Client-CipherMismatch", "UNSUPPORTED"),
    ("Resume-Server-OmitPSKsOnSecondClientHello", "UNSUPPORTED"),
    ("RetainOnlySHA256-*", "UNSUPPORTED"),
    ("SendClientVersion-RSA", "UNSUPPORTED"),
    ("SendFallbackSCSV", "UNSUPPORTED"),
    ("SendSCTListOnResume-TLS-TLS12", "UNSUPPORTED"),
    ("SendUnsolicitedOCSPOnCertificate-TLS13", "UNSUPPORTED"),
    ("SendUnsolicitedSCTOnCertificate-TLS13", "UNSUPPORTED"),
    ("SendV2ClientHello-*", "UNSUPPORTED"),
    ("Server-JDK11*", "UNSUPPORTED"),
    ("Server-VerifyDefault-ECDSA_P521_SHA512-TLS12", "UNSUPPORTED"),
    ("Server-VerifyDefault-ECDSA_P521_SHA512-TLS13", "UNSUPPORTED"),
    ("Server-VerifyDefault-Ed25519-TLS12", "UNSUPPORTED"),
    ("Server-VerifyDefault-Ed25519-TLS13", "UNSUPPORTED"),
    ("ServerBogusVersion", "UNSUPPORTED"),
    ("ServerOCSPCallback*", "UNSUPPORTED"),
    ("Shutdown-Shim-ApplicationData*", "UNSUPPORTED"),
    ("Shutdown-Shim-HelloRequest-*", "UNSUPPORTED"),
    ("Shutdown-Shim-Renegotiate-*", "UNSUPPORTED"),
    ("SignedCertificateTimestampListEmpty-Client-*", "UNSUPPORTED"),
    ("SignedCertificateTimestampListEmptySCT-Client-*", "UNSUPPORTED"),
    ("TLS-ECH-*", "UNSUPPORTED"),
    ("TLS13-Client-*TicketFlags*", "UNSUPPORTED"),
    ("TrustAnchors-*", "UNSUPPORTED"),
    ("TwoMLKEMs", "UNSUPPORTED"),
    ("VerifyPreferences-Enforced", "UNSUPPORTED"),
    ("VerifyPreferences-NoCommonAlgorithms", "UNSUPPORTED"),
  ]
  .into_iter()
  .collect()
}

fn error_map() -> HashMap<&'static str, Vector<&'static str>> {
  HashMap::new()
}

fn test_error_map() -> HashMap<&'static str, &'static str> {
  HashMap::new()
}

fn test_local_error_map() -> HashMap<&'static str, &'static str> {
  HashMap::new()
}
