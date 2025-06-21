# 🧠 Vara Signless Integration Tutorial

This repository provides a step-by-step tutorial for creating **signless integrations** on the [Vara Network](https://vara.network), allowing users to interact with smart contracts **without signing each transaction manually**.

The tutorial covers everything from adding session support in your smart contracts to deploying and using a gasless server, and finally integrating ready-to-use signless frontend components.

## ✨ What You Will Learn

- How to generate session-enabled services for signless access.
- How to configure and verify a gasless server.
- How to use AI-generated frontend components that support signless execution.

---

## 🧩 Steps to Build a Signless Integration

### 1️⃣ Add or Update Signless Support in Your Smart Contract

To enable **signless** transactions, your smart contract must include **session-based access control**.

You can either **create a new contract** or **update an existing one** using the official Vara AI Code Generator:

🔗 https://ai-codegen.vara.network/

---

### ⚙️ Update an Existing Contract

If you already have a smart contract and want to add signless support:

1. Upload your `service.rs` and `lib.rs` files to the AI generator.
2. Use the prompt:  
   > `Add signless support`
3. The AI agent will integrate the required session logic directly into your files.

---

### 🧠 Prompt Example (New Contract):

> Create a DAO service with signless support.

This will generate a contract with built-in session logic compatible with signless transactions.

---

### 🛠️ Integration Checklist:

- ✅ Add the `generate_session_system!()` macro in your main contract file.
- ✅ Include the `session_service` module in your `lib.rs`.
- ✅ Expose public service methods that validate access via session (e.g., `session_for_account`) instead of `msg::source()`.


---


### 🚀 Deploy Your Contract on Vara Network

1. Open [Gear IDEA](https://idea.gear-tech.io/programs?node=wss%3A%2F%2Frpc.vara.network) in your browser.
2. Connect your Substrate-compatible wallet (e.g. Polkadot.js extension).
3. Compile the smart contract to get the `.opt.wasm` and `.meta.txt` (IDL) files.
4. Click **Upload Program**, then select both files to deploy the contract.

Make sure the program ID is saved — you’ll need it for frontend integration and gasless voucher signing.

---

### 2️⃣ Verify a Gasless Server is Available

You’ll need a **gasless transaction relayer** compatible with Vara to handle voucher-based signless transactions.

You have several options:

- ✅ Clone and deploy the official [gasless-server-template](https://github.com/Vara-Lab/gasless-server-template).
- ✅ Use the implementation provided in this template (see the `gasless-server/` directory).
- ✅ Or build your own custom server based on your specific needs.

Be sure to configure the following environment variables:

- `PROGRAM_ID`
- `VOUCHER_ACCOUNT_SEED_HEX`
- `NODE_URL`

Also, make sure the server exposes a public HTTP endpoint capable of handling voucher-based transaction relays.

---

### 3️⃣ Generate and Use Signless Frontend Components

Use the AI code generator to create components ready to interact with the session-enabled contract:

- Components will automatically handle session creation, voucher preparation, and gasless submission.

Use the official Vara AI Code Generator:

🔗 https://ai-codegen.vara.network/

Example prompt:

> Create a React component with a signless switch and then send "Hello World".

---

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.