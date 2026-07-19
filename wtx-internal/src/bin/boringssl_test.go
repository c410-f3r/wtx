package runner

import (
	"fmt"
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
		// Not implemented
		"ALPS-", "FallbackSCSV",
		// Others
		"BadRSAClientKeyExchange-", "DelegatedCredentials-", "TLS1-",
		// Unsupported signatures
		"MinimumVersion-",
	}
	both := []string{
		// Not implemented
		"CBC", "ClientAuth", "DTLS", "ECH", "GREASE", "HRR", "QUIC",
		// Legacy
		"3DES", "ChannelID", "DSS", "MD5", "NPN", "RC4", "SHA1", "SSL3", "-TLS1-", "TLS11", "TLS12", "V2ClientHello",
		// Others
		"-HintMismatch-",
	}
	end := []string{"-TLS1"}

	individuals := []string{
		// TLS 1.2
		"NoCheckClientCertificateTypes",
		// Tls 1.3 fallback to TLS 1.2
		"ClientHelloVersionTooHigh",
		// Resumption
		"IgnoreLegacyVersion-TLS13"
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

		if tc.config.MaxVersion <= VersionTLS12 {
			printFmt(tc.name)
		} else if tc.resumeConfig != nil && tc.resumeConfig.MaxVersion <= VersionTLS12 {
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
		if strings.Contains(name, individual) {
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
	fmt.Println("    (\"" + name + "\", \"UNSUPPORTED\"),")
}