use crate::asn1::Oid;

/// 1.3.133.16.840.63.0.2
pub const OID_KDF_SHA1_SINGLE: Oid = Oid::from_bytes_opt(b"1.3.133.16.840.63.0.2").unwrap();
/// 1.3.6.1.4.1.311.2.1.4
pub const SPC_INDIRECT_DATA_OBJID: Oid = Oid::from_bytes_opt(b"1.3.6.1.4.1.311.2.1.4").unwrap();
/// 1.3.6.1.4.1.311.2.1.11
pub const SPC_STATEMENT_TYPE_OBJID: Oid = Oid::from_bytes_opt(b"1.3.6.1.4.1.311.2.1.11").unwrap();
/// 1.3.6.1.4.1.311.2.1.12
pub const SPC_SP_OPUS_INFO_OBJID: Oid = Oid::from_bytes_opt(b"1.3.6.1.4.1.311.2.1.12").unwrap();
/// 1.3.6.1.4.1.311.2.1.15
pub const SPC_PE_IMAGE_DATA: Oid = Oid::from_bytes_opt(b"1.3.6.1.4.1.311.2.1.15").unwrap();
/// 1.3.6.1.4.1.311.2.1.21
pub const SPC_INDIVIDUAL_SP_KEY_PURPOSE_OBJID : Oid = Oid::from_bytes_opt(b"1.3.6.1.4.1.311.2.1.21").unwrap();
/// 1.3.6.1.4.1.311.10.1
pub const MS_CTL: Oid = Oid::from_bytes_opt(b"1.3.6.1.4.1.311.10.1").unwrap();
/// 1.3.132.0.34
pub const OID_NIST_EC_P384: Oid = Oid::from_bytes_opt(b"1.3.132.0.34").unwrap();
/// 1.3.132.0.35
pub const OID_NIST_EC_P521: Oid = Oid::from_bytes_opt(b"1.3.132.0.35").unwrap();
/// 1.3.14.3.2.25
pub const OID_MD5_WITH_RSA: Oid = Oid::from_bytes_opt(b"1.3.14.3.2.25").unwrap();
/// 1.3.14.3.2.26
pub const OID_HASH_SHA1: Oid = Oid::from_bytes_opt(b"1.3.14.3.2.26").unwrap();
/// 1.3.14.3.2.29
pub const OID_SHA1_WITH_RSA: Oid = Oid::from_bytes_opt(b"1.3.14.3.2.29").unwrap();
/// 2.16.840.1.101.3.4.1.42
pub const OID_NIST_ENC_AES256_CBC: Oid = Oid::from_bytes_opt(b"2.16.840.1.101.3.4.1.42").unwrap();
/// 2.16.840.1.101.3.4.2.1
pub const OID_NIST_HASH_SHA256: Oid = Oid::from_bytes_opt(b"2.16.840.1.101.3.4.2.1").unwrap();
/// 2.16.840.1.101.3.4.2.2
pub const OID_NIST_HASH_SHA384: Oid = Oid::from_bytes_opt(b"2.16.840.1.101.3.4.2.2").unwrap();
/// 2.16.840.1.101.3.4.2.3
pub const OID_NIST_HASH_SHA512: Oid = Oid::from_bytes_opt(b"2.16.840.1.101.3.4.2.3").unwrap();
/// 1.2.840.113549.1.1.1
pub const OID_PKCS1_RSAENCRYPTION: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.1.1").unwrap();
/// 1.2.840.113549.1.1.2
pub const OID_PKCS1_MD2WITHRSAENC: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.1.2").unwrap();
/// 1.2.840.113549.1.1.3
pub const OID_PKCS1_MD4WITHRSAENC: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.1.3").unwrap();
/// 1.2.840.113549.1.1.4
pub const OID_PKCS1_MD5WITHRSAENC: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.1.4").unwrap();
/// 1.2.840.113549.1.1.5
pub const OID_PKCS1_SHA1WITHRSA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.1.5").unwrap();
/// 1.2.840.113549.1.1.10
pub const OID_PKCS1_RSASSAPSS: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.1.10").unwrap();
/// 1.2.840.113549.1.1.11
pub const OID_PKCS1_SHA256WITHRSA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.1.11").unwrap();
/// 1.2.840.113549.1.1.12
pub const OID_PKCS1_SHA384WITHRSA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.1.12").unwrap();
/// 1.2.840.113549.1.1.13
pub const OID_PKCS1_SHA512WITHRSA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.1.13").unwrap();
/// 1.2.840.113549.1.1.14
pub const OID_PKCS1_SHA224WITHRSA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.1.14").unwrap();
/// 1.2.840.113549.1.12
pub const OID_PKCS12: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.12").unwrap();
/// 1.2.840.113549.1.12.1
pub const OID_PKCS12_PBEIDS: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.12.1").unwrap();
/// 1.2.840.113549.1.12.1.1
pub const OID_PKCS12_PBE_SHA1_128RC4: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.12.1.1").unwrap();
/// 1.2.840.113549.1.12.1.2
pub const OID_PKCS12_PBE_SHA1_40RC4: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.12.1.2").unwrap();
/// 1.2.840.113549.1.12.1.3
pub const OID_PKCS12_PBE_SHA1_3K_3DES_CBC: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.12.1.3").unwrap();
/// 1.2.840.113549.1.12.1.4
pub const OID_PKCS12_PBE_SHA1_2K_3DES_CBC: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.12.1.4").unwrap();
/// 1.2.840.113549.1.12.1.5
pub const OID_PKCS12_PBE_SHA1_128RC2_CBC: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.12.1.5").unwrap();
/// 1.2.840.113549.1.12.1.6
pub const OID_PKCS12_PBE_SHA1_40RC2_CBC: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.12.1.6").unwrap();
/// 1.2.840.113549.1.7.1
pub const OID_PKCS7_ID_DATA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.7.1").unwrap();
/// 1.2.840.113549.1.7.2
pub const OID_PKCS7_ID_SIGNED_DATA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.7.2").unwrap();
/// 1.2.840.113549.1.7.3
pub const OID_PKCS7_ID_ENVELOPED_DATA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.7.3").unwrap();
/// 1.2.840.113549.1.7.4
pub const OID_PKCS7_ID_SIGNED_ENVELOPED_DATA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.7.4").unwrap();
/// 1.2.840.113549.1.7.5
pub const OID_PKCS7_ID_DIGESTED_DATA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.7.5").unwrap();
/// 1.2.840.113549.1.7.6
pub const OID_PKCS7_ID_ENCRYPTED_DATA: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.7.6").unwrap();
/// 1.2.840.113549.1.9.1
pub const OID_PKCS9_EMAIL_ADDRESS: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.9.1").unwrap();
/// 1.2.840.113549.1.9.2
pub const OID_PKCS9_UNSTRUCTURED_NAME: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.9.2").unwrap();
/// 1.2.840.113549.1.9.3
pub const OID_PKCS9_CONTENT_TYPE: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.9.3").unwrap();
/// 1.2.840.113549.1.9.4
pub const OID_PKCS9_ID_MESSAGE_DIGEST: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.9.4").unwrap();
/// 1.2.840.113549.1.9.5
pub const OID_PKCS9_SIGNING_TIME: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.9.5").unwrap();
/// 1.2.840.113549.1.9.7
pub const OID_PKCS9_CHALLENGE_PASSWORD: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.9.7").unwrap();
/// 1.2.840.113549.1.9.14
pub const OID_PKCS9_EXTENSION_REQUEST: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.9.14").unwrap();
/// 1.2.840.113549.1.9.15
pub const OID_PKCS9_SMIME_CAPABILITIES: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.9.15").unwrap();
/// 1.2.840.113549.1.9.20
pub const OID_PKCS9_FRIENDLY_NAME: Oid = Oid::from_bytes_opt(b"1.2.840.113549.1.9.20").unwrap();
/// 2.5
pub const OID_X500: Oid = Oid::from_bytes_opt(b"2.5").unwrap();
/// 0.9.2342.19200300.100.1.1
pub const OID_USERID: Oid = Oid::from_bytes_opt(b"0.9.2342.19200300.100.1.1").unwrap();
/// 0.9.2342.19200300.100.1.25
pub const OID_DOMAIN_COMPONENT: Oid = Oid::from_bytes_opt(b"0.9.2342.19200300.100.1.25").unwrap();
/// 1.2.643.2.2.3
pub const OID_SIG_GOST_R3411_94_WITH_R3410_2001: Oid = Oid::from_bytes_opt(b"1.2.643.2.2.3").unwrap();
/// 1.2.643.2.2.19
pub const OID_GOST_R3410_2001: Oid = Oid::from_bytes_opt(b"1.2.643.2.2.19").unwrap();
/// 1.2.643.7.1.1.1.1
pub const OID_KEY_TYPE_GOST_R3410_2012_256: Oid = Oid::from_bytes_opt(b"1.2.643.7.1.1.1.1").unwrap();
/// 1.2.643.7.1.1.1.2
pub const OID_KEY_TYPE_GOST_R3410_2012_512: Oid = Oid::from_bytes_opt(b"1.2.643.7.1.1.1.2").unwrap();
/// 1.2.643.7.1.1.3.2
pub const OID_SIG_GOST_R3410_2012_256: Oid = Oid::from_bytes_opt(b"1.2.643.7.1.1.3.2").unwrap();
/// 1.2.643.7.1.1.3.3
pub const OID_SIG_GOST_R3410_2012_512: Oid = Oid::from_bytes_opt(b"1.2.643.7.1.1.3.3").unwrap();
/// 1.2.840.10040.4.1
pub const OID_KEY_TYPE_DSA: Oid = Oid::from_bytes_opt(b"1.2.840.10040.4.1").unwrap();
/// 1.2.840.10040.4.3
pub const OID_SIG_DSA_WITH_SHA1: Oid = Oid::from_bytes_opt(b"1.2.840.10040.4.3").unwrap();
/// 1.3.36.3.3.1.2
pub const OID_SIG_RSA_RIPE_MD160: Oid = Oid::from_bytes_opt(b"1.3.36.3.3.1.2").unwrap();
/// 1.3.101.112
pub const OID_SIG_ED25519: Oid = Oid::from_bytes_opt(b"1.3.101.112").unwrap();
/// 1.3.101.113
pub const OID_SIG_ED448: Oid = Oid::from_bytes_opt(b"1.3.101.113").unwrap();
/// 1.3.6.1.4.1.311.60.2.1.1
pub const MS_JURISDICTION_LOCALITY: Oid = Oid::from_bytes_opt(b"1.3.6.1.4.1.311.60.2.1.1").unwrap();
/// 1.3.6.1.4.1.311.60.2.1.2
pub const MS_JURISDICTION_STATE_OR_PROVINCE: Oid = Oid::from_bytes_opt(b"1.3.6.1.4.1.311.60.2.1.2").unwrap();
/// 1.3.6.1.4.1.311.60.2.1.3
pub const MS_JURISDICTION_COUNTRY: Oid = Oid::from_bytes_opt(b"1.3.6.1.4.1.311.60.2.1.3").unwrap();
/// 1.3.6.1.4.1.11129.2.4.2
pub const OID_CT_LIST_SCT: Oid = Oid::from_bytes_opt(b"1.3.6.1.4.1.11129.2.4.2").unwrap();
/// 1.3.6.1.5.5.7.1.1
pub const OID_PKIX_AUTHORITY_INFO_ACCESS: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.1.1").unwrap();
/// 1.3.6.1.5.5.7.1.11
pub const OID_PKIX_SUBJECT_INFO_ACCESS: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.1.11").unwrap();
/// 1.3.6.1.5.5.7.48.1
pub const OID_PKIX_ACCESS_DESCRIPTOR_OCSP: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.1").unwrap();
/// 1.3.6.1.5.5.7.48.2
pub const OID_PKIX_ACCESS_DESCRIPTOR_CA_ISSUERS: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.2").unwrap();
/// 1.3.6.1.5.5.7.48.3
pub const OID_PKIX_ACCESS_DESCRIPTOR_TIMESTAMPING: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.3").unwrap();
/// 1.3.6.1.5.5.7.48.4
pub const OID_PKIX_ACCESS_DESCRIPTOR_DVCS: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.4").unwrap();
/// 1.3.6.1.5.5.7.48.5
pub const OID_PKIX_ACCESS_DESCRIPTOR_CA_REPOSITORY: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.5").unwrap();
/// 1.3.6.1.5.5.7.48.6
pub const OID_PKIX_ACCESS_DESCRIPTOR_HTTP_CERTS: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.6").unwrap();
/// 1.3.6.1.5.5.7.48.7
pub const OID_PKIX_ACCESS_DESCRIPTOR_HTTP_CRLS: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.7").unwrap();
/// 1.3.6.1.5.5.7.48.10
pub const OID_PKIX_ACCESS_DESCRIPTOR_RPKI_MANIFEST: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.10").unwrap();
/// 1.3.6.1.5.5.7.48.11
pub const OID_PKIX_ACCESS_DESCRIPTOR_SIGNED_OBJECT: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.11").unwrap();
/// 1.3.6.1.5.5.7.48.12
pub const OID_PKIX_ACCESS_DESCRIPTOR_CMC: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.12").unwrap();
/// 1.3.6.1.5.5.7.48.13
pub const OID_PKIX_ACCESS_DESCRIPTOR_RPKI_NOTIFY: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.13").unwrap();
/// 1.3.6.1.5.5.7.48.14
pub const OID_PKIX_ACCESS_DESCRIPTOR_STIRTNLIST: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.48.14").unwrap();
/// 2.5.4
pub const OID_X509: Oid = Oid::from_bytes_opt(b"2.5.4").unwrap();
/// 2.5.4.0
pub const OID_X509_OBJECT_CLASS: Oid = Oid::from_bytes_opt(b"2.5.4.0").unwrap();
/// 2.5.4.1
pub const OID_X509_ALIASED_ENTRY_NAME: Oid = Oid::from_bytes_opt(b"2.5.4.1").unwrap();
/// 2.5.4.2
pub const OID_X509_KNOWLEDGE_INFORMATION: Oid = Oid::from_bytes_opt(b"2.5.4.2").unwrap();
/// 2.5.4.3
pub const OID_X509_COMMON_NAME: Oid = Oid::from_bytes_opt(b"2.5.4.3").unwrap();
/// 2.5.4.4
pub const OID_X509_SURNAME: Oid = Oid::from_bytes_opt(b"2.5.4.4").unwrap();
/// 2.5.4.5
pub const OID_X509_SERIALNUMBER: Oid = Oid::from_bytes_opt(b"2.5.4.5").unwrap();
/// 2.5.4.6
pub const OID_X509_COUNTRY_NAME: Oid = Oid::from_bytes_opt(b"2.5.4.6").unwrap();
/// 2.5.4.7
pub const OID_X509_LOCALITY_NAME: Oid = Oid::from_bytes_opt(b"2.5.4.7").unwrap();
/// 2.5.4.8
pub const OID_X509_STATE_OR_PROVINCE_NAME: Oid = Oid::from_bytes_opt(b"2.5.4.8").unwrap();
/// 2.5.4.9
pub const OID_X509_STREET_ADDRESS: Oid = Oid::from_bytes_opt(b"2.5.4.9").unwrap();
/// 2.5.4.10
pub const OID_X509_ORGANIZATION_NAME: Oid = Oid::from_bytes_opt(b"2.5.4.10").unwrap();
/// 2.5.4.11
pub const OID_X509_ORGANIZATIONAL_UNIT: Oid = Oid::from_bytes_opt(b"2.5.4.11").unwrap();
/// 2.5.4.12
pub const OID_X509_TITLE: Oid = Oid::from_bytes_opt(b"2.5.4.12").unwrap();
/// 2.5.4.13
pub const OID_X509_DESCRIPTION: Oid = Oid::from_bytes_opt(b"2.5.4.13").unwrap();
/// 2.5.4.14
pub const OID_X509_SEARCH_GUIDE: Oid = Oid::from_bytes_opt(b"2.5.4.14").unwrap();
/// 2.5.4.15
pub const OID_X509_BUSINESS_CATEGORY: Oid = Oid::from_bytes_opt(b"2.5.4.15").unwrap();
/// 2.5.4.16
pub const OID_X509_POSTAL_ADDRESS: Oid = Oid::from_bytes_opt(b"2.5.4.16").unwrap();
/// 2.5.4.17
pub const OID_X509_POSTAL_CODE: Oid = Oid::from_bytes_opt(b"2.5.4.17").unwrap();
/// 2.5.4.41
pub const OID_X509_NAME: Oid = Oid::from_bytes_opt(b"2.5.4.41").unwrap();
/// 2.5.4.42
pub const OID_X509_GIVEN_NAME: Oid = Oid::from_bytes_opt(b"2.5.4.42").unwrap();
/// 2.5.4.43
pub const OID_X509_INITIALS: Oid = Oid::from_bytes_opt(b"2.5.4.43").unwrap();
/// 2.5.4.44
pub const OID_X509_GENERATION_QUALIFIER: Oid = Oid::from_bytes_opt(b"2.5.4.44").unwrap();
/// 2.5.4.45
pub const OID_X509_UNIQUE_IDENTIFIER: Oid = Oid::from_bytes_opt(b"2.5.4.45").unwrap();
/// 2.5.4.46
pub const OID_X509_DN_QUALIFIER: Oid = Oid::from_bytes_opt(b"2.5.4.46").unwrap();
/// 2.5.29.1
pub const OID_X509_OBSOLETE_AUTHORITY_KEY_IDENTIFIER: Oid = Oid::from_bytes_opt(b"2.5.29.1").unwrap();
/// 2.5.29.2
pub const OID_X509_OBSOLETE_KEY_ATTRIBUTES: Oid = Oid::from_bytes_opt(b"2.5.29.2").unwrap();
/// 2.5.29.3
pub const OID_X509_OBSOLETE_CERTIFICATE_POLICIES: Oid = Oid::from_bytes_opt(b"2.5.29.3").unwrap();
/// 2.5.29.4
pub const OID_X509_OBSOLETE_KEY_USAGE: Oid = Oid::from_bytes_opt(b"2.5.29.4").unwrap();
/// 2.5.29.5
pub const OID_X509_OBSOLETE_POLICY_MAPPING: Oid = Oid::from_bytes_opt(b"2.5.29.5").unwrap();
/// 2.5.29.6
pub const OID_X509_OBSOLETE_SUBTREES_CONSTRAINT: Oid = Oid::from_bytes_opt(b"2.5.29.6").unwrap();
/// 2.5.29.7
pub const OID_X509_OBSOLETE_SUBJECT_ALT_NAME: Oid = Oid::from_bytes_opt(b"2.5.29.7").unwrap();
/// 2.5.29.8
pub const OID_X509_OBSOLETE_ISSUER_ALT_NAME: Oid = Oid::from_bytes_opt(b"2.5.29.8").unwrap();
/// 2.5.29.14
pub const OID_X509_EXT_SUBJECT_KEY_IDENTIFIER: Oid = Oid::from_bytes_opt(b"2.5.29.14").unwrap();
/// 2.5.29.15
pub const OID_X509_EXT_KEY_USAGE: Oid = Oid::from_bytes_opt(b"2.5.29.15").unwrap();
/// 2.5.29.16
pub const OID_X509_EXT_PRIVATE_KEY_USAGE_PERIOD: Oid = Oid::from_bytes_opt(b"2.5.29.16").unwrap();
/// 2.5.29.17
pub const OID_X509_EXT_SUBJECT_ALT_NAME: Oid = Oid::from_bytes_opt(b"2.5.29.17").unwrap();
/// 2.5.29.18
pub const OID_X509_EXT_ISSUER_ALT_NAME: Oid = Oid::from_bytes_opt(b"2.5.29.18").unwrap();
/// 2.5.29.19
pub const OID_X509_EXT_BASIC_CONSTRAINTS: Oid = Oid::from_bytes_opt(b"2.5.29.19").unwrap();
/// 2.5.29.20
pub const OID_X509_EXT_CRL_NUMBER: Oid = Oid::from_bytes_opt(b"2.5.29.20").unwrap();
/// 2.5.29.21
pub const OID_X509_EXT_REASON_CODE: Oid = Oid::from_bytes_opt(b"2.5.29.21").unwrap();
/// 2.5.29.23
pub const OID_X509_EXT_HOLD_INSTRUCTION_CODE: Oid = Oid::from_bytes_opt(b"2.5.29.23").unwrap();
/// 2.5.29.24
pub const OID_X509_EXT_INVALIDITY_DATE: Oid = Oid::from_bytes_opt(b"2.5.29.24").unwrap();
/// 2.5.29.27
pub const OID_X509_EXT_DELTA_CRL_INDICATOR: Oid = Oid::from_bytes_opt(b"2.5.29.27").unwrap();
/// 2.5.29.28
pub const OID_X509_EXT_ISSUER_DISTRIBUTION_POINT: Oid = Oid::from_bytes_opt(b"2.5.29.28").unwrap();
/// 2.5.29.29
pub const OID_X509_EXT_ISSUER: Oid = Oid::from_bytes_opt(b"2.5.29.29").unwrap();
/// 2.5.29.30
pub const OID_X509_EXT_NAME_CONSTRAINTS: Oid = Oid::from_bytes_opt(b"2.5.29.30").unwrap();
/// 2.5.29.31
pub const OID_X509_EXT_CRL_DISTRIBUTION_POINTS: Oid = Oid::from_bytes_opt(b"2.5.29.31").unwrap();
/// 2.5.29.32
pub const OID_X509_EXT_CERTIFICATE_POLICIES: Oid = Oid::from_bytes_opt(b"2.5.29.32").unwrap();
/// 2.5.29.33
pub const OID_X509_EXT_POLICY_MAPPINGS: Oid = Oid::from_bytes_opt(b"2.5.29.33").unwrap();
/// 2.5.29.35
pub const OID_X509_EXT_AUTHORITY_KEY_IDENTIFIER: Oid = Oid::from_bytes_opt(b"2.5.29.35").unwrap();
/// 2.5.29.36
pub const OID_X509_EXT_POLICY_CONSTRAINTS: Oid = Oid::from_bytes_opt(b"2.5.29.36").unwrap();
/// 2.5.29.37
pub const OID_X509_EXT_EXTENDED_KEY_USAGE: Oid = Oid::from_bytes_opt(b"2.5.29.37").unwrap();
/// 2.5.29.46
pub const OID_X509_EXT_FRESHEST_CRL: Oid = Oid::from_bytes_opt(b"2.5.29.46").unwrap();
/// 2.5.29.54
pub const OID_X509_EXT_INHIBIT_ANY_POLICY: Oid = Oid::from_bytes_opt(b"2.5.29.54").unwrap();
/// 2.16.840.1.113730.1.1
pub const OID_X509_EXT_CERT_TYPE: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.1").unwrap();
/// 2.16.840.1.113730.1.2
pub const OID_X509_EXT_BASE_URL: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.2").unwrap();
/// 2.16.840.1.113730.1.3
pub const OID_X509_EXT_REVOCATION_URL: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.3").unwrap();
/// 2.16.840.1.113730.1.4
pub const OID_X509_EXT_CA_REVOCATION_URL: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.4").unwrap();
/// 2.16.840.1.113730.1.5
pub const OID_X509_EXT_CA_CRL_URL: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.5").unwrap();
/// 2.16.840.1.113730.1.6
pub const OID_X509_EXT_CA_CERT_URL: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.6").unwrap();
/// 2.16.840.1.113730.1.7
pub const OID_X509_EXT_RENEWAL_URL: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.7").unwrap();
/// 2.16.840.1.113730.1.8
pub const OID_X509_EXT_CA_POLICY_URL: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.8").unwrap();
/// 2.16.840.1.113730.1.9
pub const OID_X509_EXT_HOMEPAGE_URL: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.9").unwrap();
/// 2.16.840.1.113730.1.10
pub const OID_X509_EXT_ENTITY_LOGO: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.10").unwrap();
/// 2.16.840.1.113730.1.11
pub const OID_X509_EXT_USER_PICTURE: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.11").unwrap();
/// 2.16.840.1.113730.1.12
pub const OID_X509_EXT_SSL_SERVER_NAME: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.12").unwrap();
/// 2.16.840.1.113730.1.13
pub const OID_X509_EXT_CERT_COMMENT: Oid = Oid::from_bytes_opt(b"2.16.840.1.113730.1.13").unwrap();
/// 1.2.840.10045.2.1
pub const OID_KEY_TYPE_EC_PUBLIC_KEY: Oid = Oid::from_bytes_opt(b"1.2.840.10045.2.1").unwrap();
/// 1.2.840.10045.4.3.1
pub const OID_SIG_ECDSA_WITH_SHA224: Oid = Oid::from_bytes_opt(b"1.2.840.10045.4.3.1").unwrap();
/// 1.2.840.10045.4.3.2
pub const OID_SIG_ECDSA_WITH_SHA256: Oid = Oid::from_bytes_opt(b"1.2.840.10045.4.3.2").unwrap();
/// 1.2.840.10045.4.3.3
pub const OID_SIG_ECDSA_WITH_SHA384: Oid = Oid::from_bytes_opt(b"1.2.840.10045.4.3.3").unwrap();
/// 1.2.840.10045.4.3.4
pub const OID_SIG_ECDSA_WITH_SHA512: Oid = Oid::from_bytes_opt(b"1.2.840.10045.4.3.4").unwrap();
/// 1.2.840.10045.3.1.7
pub const OID_EC_P256: Oid = Oid::from_bytes_opt(b"1.2.840.10045.3.1.7").unwrap();
/// 1.3.101.110
pub const OID_X25519: Oid = Oid::from_bytes_opt(b"1.3.101.110").unwrap();
/// 2.5.29.37.0
pub const OID_X509_EXT_ANY_EXTENDED_KEY_USAGE: Oid = Oid::from_bytes_opt(b"2.5.29.37.0").unwrap();
/// 1.3.6.1.5.5.7.3.1
pub const OID_PKIX_KP_SERVER_AUTH: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.3.1").unwrap();
/// 1.3.6.1.5.5.7.3.2
pub const OID_PKIX_KP_CLIENT_AUTH: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.3.2").unwrap();
/// 1.3.6.1.5.5.7.3.3
pub const OID_PKIX_KP_CODE_SIGNING: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.3.3").unwrap();
/// 1.3.6.1.5.5.7.3.4
pub const OID_PKIX_KP_EMAIL_PROTECTION: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.3.4").unwrap();
/// 1.3.6.1.5.5.7.3.8
pub const OID_PKIX_KP_TIMESTAMPING: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.3.8").unwrap();
/// 1.3.6.1.5.5.7.3.9
pub const OID_PKIX_KP_OCSP_SIGNING: Oid = Oid::from_bytes_opt(b"1.3.6.1.5.5.7.3.9").unwrap();

