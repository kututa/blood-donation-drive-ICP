# Blood Donation drive

This is a rust smart contract that facilitates the management of a blood donation drive, allowing hospitals, patients, and donors to participate. It provides functions for adding entities, querying information, and handling pledges. The code incorporates validation and clear error messages to ensure the integrity of the data and the security of the smart contract.

## installation

```bash
git clone https://github.com/kututa/blood-donation-drive-ICP.git
cd blood-donation-drive-ICP
npm install
dfx start --background --clean
npm run gen-deploy
```

## Structure

### Struct Definitions

1. **Patient:**
   - Represents a patient with attributes such as ID, name, blood group, hospital, description, needed pints, donations, password, and completion status.

2. **Hospital:**
   - Represents a hospital with attributes including ID, name, address, password, city, donations, and donor IDs.

3. **Donor:**
   - Represents a donor with attributes like ID, name, password, blood group, and beneficiaries (IDs of patients they pledged to).

### Storable and BoundedStorable Implementations

- Implements the `Storable` and `BoundedStorable` traits for the `Patient`, `Hospital`, and `Donor` structs, enabling serialization and deserialization.

### Memory Management and Storage

- Utilizes a thread-local static variable for a `MemoryManager` and `IdCell` for managing memory and generating unique IDs.
- Uses `StableBTreeMap` for storing patients, hospitals, and donors in stable memory.

### Payload Structs

1. **HospitalPayload:**
   - Payload structure for adding a new hospital.

2. **PatientPayload:**
   - Payload structure for adding a new patient.

3. **EditPatientPayload:**
   - Payload structure for editing patient attributes.

4. **DonorPayload:**
   - Payload structure for adding a new donor.

5. **EditHospitalPayload:**
   - Payload structure for editing hospital attributes.

6. **PledgePayload:**
   - Payload structure for a donor pledging to a hospital or patient.

### Query Functions

1. **get_all_hospitals:**
   - Retrieves all hospitals.

2. **get_hospital_by_city_and_name:**
   - Retrieves hospitals by city or name.

3. **get_hospital_by_id:**
   - Retrieves a hospital by ID.

4. **get_patient:**
   - Retrieves a patient by ID.

5. **get_incomplete_donation_patients:**
   - Retrieves incomplete donation patients.

### Update Functions

1. **add_hospital:**
   - Adds a new hospital.

2. **edit_hospital:**
   - Edits hospital attributes.

3. **pledge_to_hospital:**
   - Handles a donor pledging to a hospital.

4. **add_patient:**
   - Adds a new patient.

5. **edit_patient:**
   - Edits patient attributes.

6. **pledge_to_patient:**
   - Handles a donor pledging to a patient.

### Error Handling

- Defines an `Error` enum for handling various error scenarios like not found, already initialized, invalid payload, and unauthorized access.

### Candid Interface

- Exports the Candid interface for seamless interaction with the Internet Computer.
  
## More

To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

To learn more before you start working with recipe_nft, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.icp0.io)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd recipe_nft/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.

### Note on frontend environment variables

If you are hosting frontend code somewhere without using DFX, you may need to make one of the following adjustments to ensure your project does not fetch the root key in production:

- set`DFX_NETWORK` to `production` if you are using Webpack
- use your own preferred method to replace `process.env.DFX_NETWORK` in the autogenerated declarations
  - Setting `canisters -> {asset_canister_id} -> declarations -> env_override to a string` in `dfx.json` will replace `process.env.DFX_NETWORK` with the string in the autogenerated declarations
- Write your own `createActor` constructor
