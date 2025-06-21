# 🧩 Vara Signless + Payless Template

A developer-friendly React + TypeScript template to quickly integrate **signless** and **payless** transactions on the [Vara Network](https://vara.network).

---

## 🚀 Quick Integration Steps

Easily integrate **signless** transactions into your Vara Network dApp using this template.

---

### ✅ Step 1: Generate Your Signless Component with the AI Generator

Use the official Vara AI Code Generator:

🔗 https://ai-codegen.vara.network/

Example prompt:

> Create a React component with a signless switch and then send "Hello World".

---

### ✅ Step 2: Add Your `lib.ts` File in `hocs/`

Make sure to place your logic handler (`lib.ts`) inside the `hocs/` directory.

---

### ✅ Step 3: Add Your Signless Component

Save the generated component (e.g., `SwitchSignlessAndSendHello.tsx`) inside a folder like `src/home/` or `src/components/`.

---

### ✅ Step 4: Import Your Signless Component

```tsx
import { EzSwitchAndSendHello } from "./EzSwitchAndSendHello";
import { SwitchSignlessAndSendHello } from "./SwitchSignless";

function Home() {
  return <EzSwitchAndSendHello/>
}

export { Home };

```
### ✅ Step 5: Initialize the Template


```bash
yarn install

yarn start
```

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.