use crate::Result;

use rcgen::{
    BasicConstraints, Certificate, CertificateParams, CertificateSigningRequest, DnType, IsCa,
};
use x509_parser::certification_request::X509CertificationRequest;
use x509_parser::prelude::*;

pub struct SelfSignedSet {
    pub ca: Ca,
    pub entity: Entity,

    pub ca_pem: String,
    pub csr_pem: String,
    pub direct: String,
    pub indirect: String,
    pub key: Vec<u8>,
}

pub(crate) fn generate_and_sign(
    common_name: &str,
    subject_alt_names: impl Into<Vec<String>> + Clone,
) -> Result<SelfSignedSet> {
    let ca = Ca::new(common_name, subject_alt_names.clone());
    let entity = Entity::new(common_name, subject_alt_names);
    let csr = entity.create_csr();
    let direct = entity
        .certificate
        .serialize_pem_with_signer(&ca.certificate)?;
    let indirect = ca.create_cert(&csr);
    let key = entity.certificate.serialize_private_key_der();
    let ca_pem = ca.certificate.serialize_pem()?;
    let cert_set = SelfSignedSet {
        ca,
        entity,

        ca_pem,
        csr_pem: csr,
        direct,
        indirect,
        key,
    };

    Ok(cert_set)
}

pub(crate) struct Ca {
    pub certificate: Certificate,
}

impl Ca {
    fn new(common_name: &str, subject_alt_names: impl Into<Vec<String>>) -> Self {
        let mut params = CertificateParams::new(subject_alt_names);
        params
            .distinguished_name
            .push(DnType::CommonName, common_name);
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        Self {
            certificate: Certificate::from_params(params).unwrap(),
        }
    }

    fn create_cert(&self, csr_pem: &str) -> String {
        let csr_der = x509_parser::pem::parse_x509_pem(csr_pem.as_bytes())
            .unwrap()
            .1;
        let csr = X509CertificationRequest::from_der(&csr_der.contents)
            .unwrap()
            .1;
        csr.verify_signature().unwrap();
        let csr = CertificateSigningRequest::from_der(&csr_der.contents).unwrap();
        csr.serialize_pem_with_signer(&self.certificate).unwrap()
    }
}

pub struct Entity {
    certificate: Certificate,
}

impl Entity {
    fn new(common_name: &str, subject_alt_names: impl Into<Vec<String>>) -> Self {
        let mut params = CertificateParams::new(subject_alt_names);
        params
            .distinguished_name
            .push(DnType::CommonName, common_name);
        Self {
            certificate: Certificate::from_params(params).unwrap(),
        }
    }

    fn create_csr(&self) -> String {
        self.certificate.serialize_request_pem().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_sign_verify() {
        let common_name = "example.com";
        let subject_alt_names: Vec<String> = vec![
            "example.com".into(),
            "self-signed.example.com".into(),
            "jfaawekmawdvawf.example.com".into(),
        ];

        let _ = generate_and_sign(common_name, subject_alt_names).unwrap();
    }
}
