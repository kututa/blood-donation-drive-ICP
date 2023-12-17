#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use validator::Validate;

// Define type aliases for convenience
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Patient {
    id: u64,
    name: String,
    blood_group: String,
    hospital: String,
    description: String,
    needed_pints: u32,
    donations: u32,
    password: String,
    is_complete: bool,
    donors_ids: Vec<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Hospital {
    id: u64,
    name: String,
    address: String,
    password: String,
    city: String,
    donations: u32,
    donors_ids: Vec<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Donor {
    id: u64,
    name: String,
    password: String,
    blood_group: String,
    beneficiaries: Vec<u64>,
}

// Implement the 'Storable' trait for 'Hospital', 'Patient' and 'CommunityHospital'

impl Storable for Patient {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Hospital {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Donor {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement the 'BoundedStorable' trait for 'Hospital', 'Patient' and 'CommunityHospital'
impl BoundedStorable for Patient {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Hospital {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Donor {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Define thread-local static variables for memory management and storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static PATIENT_STORAGE: RefCell<StableBTreeMap<u64, Patient, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static HOSPITAL_STORAGE: RefCell<StableBTreeMap<u64, Hospital, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static DONOR_STORAGE: RefCell<StableBTreeMap<u64, Donor, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
    ));
}

// Struct for payload date used in update functions
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct HospitalPayload {
    #[validate(length(min = 3))]
    name: String,
    #[validate(length(min = 3))]
    address: String,
    password: String,
    city: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct PatientPayload {
    #[validate(length(min = 3))]
    name: String,
    blood_group: String,
    #[validate(length(min = 6))]
    description: String,
    password: String,
    hospital: String,
    needed_pints: u32,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct EditPatientPayload {
    patient_id: u64,
    needed_pints: u32,
    password: String,
    is_complete: bool,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct DonorPayload {
    #[validate(length(min = 3))]
    name: String,
    blood_group: String,
    #[validate(length(min = 4))]
    password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct EditHospitalPayload {
    hospital_id: u64,
    name: String,
    password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct PledgePayload {
    donor_id: u64,
    recipient_id: u64,
    pints_pledge: u32,
    password: String,
}

// Query function to get all hospitals
#[ic_cdk::query]
fn get_all_hospitals() -> Result<Vec<Hospital>, Error> {
    // Retrieve all Hospitals from the storage
    let hospital_map: Vec<(u64, Hospital)> = HOSPITAL_STORAGE.with(|s| s.borrow().iter().collect());
    // Extract the Hospitals from the tuple and create a vector
    let hospitals: Vec<Hospital> = hospital_map
        .into_iter()
        .map(|(_, hospital)| hospital)
        .map(|hospital| Hospital {
            password: "******".to_string(),
            ..hospital
        })
        .collect();

    match hospitals.len() {
        0 => Err(Error::NotFound {
            msg: format!("no Hospitals found"),
        }),
        _ => Ok(hospitals),
    }
}

// Get Hospitals by city and name content
#[ic_cdk::query]
fn get_hospital_by_city_and_name(search: String) -> Result<Vec<Hospital>, Error> {
    let query = search.to_lowercase();
    // Retrieve all Hospitals from the storage
    let hospital_map: Vec<(u64, Hospital)> = HOSPITAL_STORAGE.with(|s| s.borrow().iter().collect());
    // Extract the Hospitals from the tuple and create a vector
    let hospitals: Vec<Hospital> = hospital_map
        .into_iter()
        .map(|(_, hospital)| hospital)
        .collect();

    // Filter the hospitals by city or name
    let incomplete_patients: Vec<Hospital> = hospitals
        .into_iter()
        .filter(|hospital| {
            (hospital.city).to_lowercase().contains(&query)
                || (hospital.name).to_lowercase().contains(&query)
        })
        .map(|hospital| Hospital {
            password: "******".to_string(),
            ..hospital
        })
        .collect();

    // Check if any hospitals are found
    match incomplete_patients.len() {
        0 => Err(Error::NotFound {
            msg: format!(
                "no Food hospitals for city or name: {} could be found",
                query
            ),
        }),
        _ => Ok(incomplete_patients),
    }
}

// get hospital by ID
#[ic_cdk::query]
fn get_hospital_by_id(id: u64) -> Result<Hospital, Error> {
    match HOSPITAL_STORAGE.with(|hospitals| hospitals.borrow().get(&id)) {
        Some(hospital) => Ok(Hospital {
            password: "******".to_string(),
            ..hospital
        }),
        None => Err(Error::NotFound {
            msg: format!("hospital of id: {} not found", id),
        }),
    }
}

// Create new Hospital
#[ic_cdk::update]
fn add_hospital(payload: HospitalPayload) -> Result<Hospital, Error> {
    // validate payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let hospital = Hospital {
        id,
        name: payload.name.clone(),
        address: payload.address,
        city: payload.city,
        password: payload.password,
        donations: 0,
        donors_ids: vec![],
    };

    match HOSPITAL_STORAGE.with(|s| s.borrow_mut().insert(id, hospital.clone())) {
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("Could not add hospital name: {}", payload.name),
        }),
        None => Ok(hospital),
    }
}

// update function to edit a hospital where only owners of hospitals can edit title, is_community, price and description. Non owners can only edit descriptions of communtiy hospitals. authorizations is by password
#[ic_cdk::update]
fn edit_hospital(payload: EditHospitalPayload) -> Result<Hospital, Error> {
    let hospital = HOSPITAL_STORAGE.with(|hospitals| hospitals.borrow().get(&payload.hospital_id));

    match hospital {
        Some(hospital) => {
            // check if the password provided matches hospital
            if hospital.password != payload.password {
                return Err(Error::Unauthorized {
                    msg: format!("Unauthorized, password does not match, try again"),
                });
            }

            let new_hospital = Hospital {
                name: payload.name,
                ..hospital.clone()
            };

            match HOSPITAL_STORAGE
                .with(|s| s.borrow_mut().insert(hospital.id, new_hospital.clone()))
            {
                Some(_) => Ok(new_hospital),
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not edit hospital title: {}", hospital.name),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("hospital of id: {} not found", payload.hospital_id),
        }),
    }
}

// function to pledge to hospital
#[ic_cdk::update]
fn pledge_to_hospital(payload: PledgePayload) -> Result<String, Error> {
    // get hospital
    let hospital = HOSPITAL_STORAGE.with(|hospitals| hospitals.borrow().get(&payload.recipient_id));
    match hospital {
        Some(hospital) => {
            // check if the password provided matches hospital
            if hospital.password != payload.password {
                return Err(Error::Unauthorized {
                    msg: format!("Unauthorized, password does not match, try again"),
                });
            }

            // check if donor has enough balance
            let donor = DONOR_STORAGE.with(|donors| donors.borrow().get(&payload.donor_id));
            match donor {
                Some(donor) => {
                    let mut new_donor_beneficiaries = donor.beneficiaries.clone();
                    new_donor_beneficiaries.push(hospital.id);
                    let new_donor = Donor {
                        beneficiaries: new_donor_beneficiaries,
                        ..donor
                    };
                    // update donor in storage
                    match DONOR_STORAGE.with(|s| s.borrow_mut().insert(donor.id, new_donor.clone()))
                    {
                        Some(_) => {
                            // update hospital
                            let mut new_hospital_donors_ids = hospital.donors_ids.clone();
                            new_hospital_donors_ids.push(donor.id);
                            let new_hospital = Hospital {
                                donors_ids: new_hospital_donors_ids,
                                ..hospital.clone()
                            };
                            // update hospital in storage
                            match HOSPITAL_STORAGE
                                .with(|s| s.borrow_mut().insert(hospital.id, new_hospital.clone()))
                            {
                                Some(_) => Ok(format!(
                                    "Succesfully pledged to hospital {}, visit address: {} to donate",
                                    hospital.name, hospital.address
                                )),
                                None => Err(Error::InvalidPayload {
                                    msg: format!("Could not update hospital"),
                                }),
                            }
                        }
                        None => Err(Error::InvalidPayload {
                            msg: format!("Could not update donor"),
                        }),
                    }
                }
                None => Err(Error::NotFound {
                    msg: format!("Donor of id: {} not found", payload.donor_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("hospital of id: {} not found", payload.recipient_id),
        }),
    }
}

// Define query function to get a patient by ID
#[ic_cdk::query]
fn get_patient(id: u64) -> Result<Patient, Error> {
    match PATIENT_STORAGE.with(|patients| patients.borrow().get(&id)) {
        Some(patient) => Ok(Patient {
            password: "******".to_string(),
            ..patient
        }),
        None => Err(Error::NotFound {
            msg: format!("patient id:{} does not exist", id),
        }),
    }
}

// Query function to get all for incomplete donations patients
#[ic_cdk::query]
fn get_incomplete_donation_patients() -> Result<Vec<Patient>, Error> {
    // Retrieve all Patients from the storage
    let patients_map: Vec<(u64, Patient)> = PATIENT_STORAGE.with(|s| s.borrow().iter().collect());
    // Extract the Patients from the tuple and create a vector
    let patients: Vec<Patient> = patients_map
        .into_iter()
        .map(|(_, patient)| patient)
        .collect();

    // Filter the patients by category
    let incomplete_patients: Vec<Patient> = patients
        .into_iter()
        .filter(|patient| !patient.is_complete)
        .collect();

    // convert to Patient struct
    let return_patients: Vec<Patient> = incomplete_patients
        .into_iter()
        .map(|patient| Patient {
            password: "******".to_string(),
            ..patient
        })
        .collect();

    // Check if any patients are found
    match return_patients.len() {
        0 => Err(Error::NotFound {
            msg: format!("No patients for donations could be found"),
        }),
        _ => Ok(return_patients),
    }
}

// Update function to add a patient
#[ic_cdk::update]
fn add_patient(payload: PatientPayload) -> Result<Patient, Error> {
    // validate payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let patient = Patient {
        id,
        name: payload.name.clone(),
        description: payload.description,
        blood_group: payload.blood_group,
        needed_pints: payload.needed_pints,
        hospital: payload.hospital,
        donations: 0,
        password: payload.password,
        is_complete: false,
        donors_ids: vec![],
    };

    match PATIENT_STORAGE.with(|s| s.borrow_mut().insert(id, patient.clone())) {
        None => Ok(patient),
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("Could not add patient name: {}", payload.name),
        }),
    }
}

// update function to edit a patient where authorizations is by password
#[ic_cdk::update]
fn edit_patient(payload: EditPatientPayload) -> Result<Patient, Error> {
    let patient = PATIENT_STORAGE.with(|patients| patients.borrow().get(&payload.patient_id));

    match patient {
        Some(patient) => {
            // check if the password provided matches patient
            if patient.password != payload.password {
                return Err(Error::Unauthorized {
                    msg: format!("Unauthorized, password does not match, try again"),
                });
            }

            let new_patient = Patient {
                needed_pints: payload.needed_pints,
                is_complete: payload.is_complete,
                ..patient.clone()
            };

            match PATIENT_STORAGE.with(|s| s.borrow_mut().insert(patient.id, new_patient.clone())) {
                Some(_) => Ok(new_patient),
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not edit patient name: {}", patient.name),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("patient of id: {} not found", payload.patient_id),
        }),
    }
}

// function to pledge to patient
#[ic_cdk::update]
fn pledge_to_patient(payload: PledgePayload) -> Result<String, Error> {
    // get patient
    let patient = PATIENT_STORAGE.with(|patients| patients.borrow().get(&payload.recipient_id));
    match patient {
        Some(patient) => {
            // check if the password provided matches patient
            if patient.password != payload.password {
                return Err(Error::Unauthorized {
                    msg: format!("Unauthorized, password does not match, try again"),
                });
            }

            // check if donor has enough balance
            let donor = DONOR_STORAGE.with(|donors| donors.borrow().get(&payload.donor_id));
            match donor {
                Some(donor) => {
                    if patient.donations >= patient.needed_pints {
                        return Err(Error::InvalidPayload {
                            msg: format!(
                                "Patient has already reached their needed donation target"
                            ),
                        });
                    }
                    let mut new_donor_beneficiaries = donor.beneficiaries.clone();
                    new_donor_beneficiaries.push(patient.id);
                    let new_donor = Donor {
                        beneficiaries: new_donor_beneficiaries,
                        ..donor
                    };
                    // update donor in storage
                    match DONOR_STORAGE.with(|s| s.borrow_mut().insert(donor.id, new_donor.clone()))
                    {
                        Some(_) => {
                            // update patient
                            let mut new_patient_donors_ids = patient.donors_ids.clone();
                            let is_complete = patient.needed_pints <= (patient.donations + payload.pints_pledge);
                            new_patient_donors_ids.push(donor.id);
                            let new_patient = Patient {
                                donors_ids: new_patient_donors_ids, 
                                is_complete,
                                donations: patient.donations + payload.pints_pledge,
                                ..patient.clone()
                            };
                            // update patient in storage
                            match PATIENT_STORAGE
                                .with(|s| s.borrow_mut().insert(patient.id, new_patient.clone()))
                            {
                                Some(_) => Ok(format!(
                                    "Succesfully pledged to patient {}, visit hospital: {} to donate",
                                    patient.name, patient.hospital
                                )),
                                None => Err(Error::InvalidPayload {
                                    msg: format!("Could not update patient"),
                                }),
                            }
                        }
                        None => Err(Error::InvalidPayload {
                            msg: format!("Could not update donor"),
                        }),
                    }
                }
                None => Err(Error::NotFound {
                    msg: format!("Donor of id: {} not found", payload.donor_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("patient of id: {} not found", payload.recipient_id),
        }),
    }
}

// add donor
#[ic_cdk::update]
fn add_donor(payload: DonorPayload) -> Result<Donor, Error> {
    // validate payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let donor = Donor {
        id,
        name: payload.name.clone(),
        blood_group: payload.blood_group,
        password: payload.password,
        beneficiaries: vec![],
    };

    match DONOR_STORAGE.with(|s| s.borrow_mut().insert(id, donor.clone())) {
        None => Ok(donor),
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("Could not add donor name: {}", payload.name),
        }),
    }
}

// get donor by ID
#[ic_cdk::query]
fn get_donor_by_id(id: u64) -> Result<Donor, Error> {
    match DONOR_STORAGE.with(|donors| donors.borrow().get(&id)) {
        Some(donor) => Ok(Donor {
            password: "******".to_string(),
            ..donor
        }),
        None => Err(Error::NotFound {
            msg: format!("donor id:{} does not exist", id),
        }),
    }
}

// Define an Error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    AlreadyInit { msg: String },
    InvalidPayload { msg: String },
    Unauthorized { msg: String },
}

// Candid generator for exporting the Candid interface
ic_cdk::export_candid!();
