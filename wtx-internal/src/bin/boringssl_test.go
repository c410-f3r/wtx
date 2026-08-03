package runner

import (
	"fmt"
	"slices"
	"strings"
	"testing"
)

func TestNames(t *testing.T) {
	initKeys()
	initCertificates()
	initDelegatedCredentials()
	initRawPublicKeyCredentials()

	addBasicTests()
	addCipherSuiteTests()
	addBadECDSASignatureTests()
	addCBCPaddingTests()
	addCBCSplittingTests()
	addClientAuthTests()
	addDDoSCallbackTests()
	addVersionNegotiationTests()
	addMinimumVersionTests()
	addExtensionTests()
	addResumptionVersionTests()
	addExtendedMasterSecretTests()
	addRenegotiationTests()
	addDTLSReplayTests()
	addSignatureAlgorithmTests()
	addDTLSRetransmitTests()
	addDTLSReorderTests()
	addDTLSFragmentWindowTests()
	addExportKeyingMaterialTests()
	addExportTrafficSecretsTests()
	addTLSUniqueTests()
	addUnknownExtensionTests()
	addRSAClientKeyExchangeTests()
	addCurveTests()
	addSessionTicketTests()
	addTLS13RecordTests()
	addAllStateMachineCoverageTests()
	addChangeCipherSpecTests()
	addEndOfFlightTests()
	addWrongMessageTypeTests()
	addTrailingMessageDataTests()
	addTLS13HandshakeTests()
	addTLS13CipherPreferenceTests()
	addPeekTests()
	addRecordVersionTests()
	addCertificateTests()
	addRetainOnlySHA256ClientCertTests()
	addECDSAKeyUsageTests()
	addRSAKeyUsageTests()
	addExtraHandshakeTests()
	addOmitExtensionsTests()
	addExtensionTrailingDataTests()
	addCertCompressionTests()
	addJDK11WorkaroundTests()
	addDelegatedCredentialTests()
	addEncryptedClientHelloTests()
	addHintMismatchTests()
	addCompliancePolicyTests()
	addCertificateSelectionTests()
	addKeyUpdateTests()
	addPAKETests()
	addTrustAnchorTests()
	addPSKTests()
	addRawPublicKeyTests()
	addServerPaddingTests()

	begin := []string{
		// ALPS is not implemented
		"ExtraClientEncryptedExtension-",
		// Not implemented
		"FallbackSCSV",
		// Pre Shared Key is not implemented
		"PSK-",
		// SCT is not implemented
		"SignedCertificateTimestampListEmpty",
		// OCSP is deprecated
		"UnsolicitedCertificateExtensions-",
		
		// Legacy
		"TLS1-",
		// Others
		"BadRSAClientKeyExchange-", "DelegatedCredentials-",
		// Unsupported signatures
		"MinimumVersion-",
	}
	both := []string{
		// ALPS is not implemented
		"ALPS",
		// Certificate compression in not implemented
		"CertCompression",
		// mTLS is not implemented
		"ClientAuth", "UnknownExtensionInCertificateRequest-TLS13", "AlwaysSelectPSKIdentity-TLS13",
		// CBC is not implemented
		"CBC",
		// DTLS is not implemented
		"DTLS",
		// 0-RTT is not supported
		"EarlyData",
		// ECH is not implemented
		"ECH",
		// Grease is not implemented
		"GREASE",
		// Hello Retry Request is not implemented
		"HelloRetryRequest", "PartialClientFinishedWithSecondClientHello", "SecondClientHello", "SecondServerHello",
		// HRR is not implemented
		"HRR",
		// OCSP is deprecated
		"OCSP",
		// PAKE is not implemented
		"PAKE",
		// QUIC is not implemented
		"QUIC",
		// Post Quantum is not implemented
		"MLKEM", "ML-DSA-", "PostQuantum", "cnsa1-202603", "cnsa2-202603",
		// Raw public key is not implemented
		"RawPublicKey", "RPK",
		// Resumption
		"Resumption",
		// Ticket Request is not implemented
		"TicketFlags",

		// Unrelated
		"JDK11",

		// Unsupported signatures
		"ECDSA_P521", "RSA_PKCS1", "RSA_PSS_SHA512",
		// Unsupported key exchanges
		"Kyber", "P-521",

		// Legacy
		"3DES", "ChannelID", "DSS", "ExtendedMasterSecret", "MD5", "NPN", "RC4", "SHA1", "SSL3", "-TLS1-", "TLS11", "TLS12", "V2ClientHello",
		// Others
		"-HintMismatch-",
	}
	end := []string{"-TLS1"}

	individuals := []string{
		// It is necessary to first send them to actually eval the tests, which isn't optimal because WTX does not support TLS 1.2
		"EMS-Forbidden-TLS13", "RenegotiationInfo-Forbidden-TLS13",
		// BOGO expects a graceful shutdown. WTX abruptly closes the connection
		"StrayHelloRequest-TLS13",
		// Ed25519 is already enabled by default
		"Client-VerifyDefault-Ed25519-TLS13",
		// Legacy extensions are ignored, regardless if they are valid or not
		"PointFormat-EncryptedExtensions-TLS13",
		// OCSP is deprecated
		"SendNoExtensionsOnIntermediate-TLS13",
		// SCT is not implemented
		"SendUnsolicitedSCTOnCertificate-TLS13",
		// Legacy
		"ClientHelloVersionTooHigh", "NoCheckClientCertificateTypes",
		// Resumption
		"IgnoreLegacyVersion-TLS13",
		// In a TLS 1.3 only implementation, even TLS 1.2 extensions are unknown.
		//
		// AFAICT, the only way to differentiate between legacy and actual unknowns is to support
		// legacy extensions identifiers, which doesn't seem worthwhile.
		"UnknownExtension-Client-TLS13", "UnknownUnencryptedExtension-Client-TLS13",
	}

	for _, tc := range testCases {
		if tc.config.MinVersion == VersionTLS13 {
			continue
		}
		if hasPattern(tc.name, begin, both, end) {
			continue
		}
		if hasIndividual(tc.name, individuals) {
			continue
		}

		shouldIgnore := false
		if tc.resumeSession {
			shouldIgnore = true
		} else if tc.config.MaxVersion <= VersionTLS12 {
			shouldIgnore = true
		} else if tc.resumeConfig != nil && tc.resumeConfig.MaxVersion <= VersionTLS12 {
			shouldIgnore = true
		} else if tc.config.ClientAuth != 0 {
			shouldIgnore = true
		} else if slices.Contains(tc.flags, "-require-any-client-certificate") {
			shouldIgnore = true
		}
		if shouldIgnore {
			printFmt(tc.name)
		}
	}

	for _, pattern := range begin {
		printFmt(pattern + "*")
	}
	for _, pattern := range both {
		printFmt("*" + pattern + "*")
	}
	for _, pattern := range end {
		printFmt("*" + pattern)
	}
	for _, individual := range individuals {
		printFmt(individual)
	}

	t.Fail()
}

func hasIndividual(name string, individuals []string) bool {
	for _, individual := range individuals {
		if name == individual {
			return true
		}
	}
	return false
}

func hasPattern(name string, begin, both, end []string) bool {
	for _, pattern := range begin {
		if strings.HasPrefix(name, pattern) {
			return true
		}
	}
	for _, pattern := range both {
		if strings.Contains(name, pattern) {
			return true
		}
	}
	for _, pattern := range end {
		if strings.HasSuffix(name, pattern) {
			return true
		}
	}
	return false
}

func printFmt(name string) {
	fmt.Printf("    (%q, \"UNSUPPORTED\"),\n", name)
}