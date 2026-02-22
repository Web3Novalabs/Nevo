# Installation Instructions

Install the Stellar SDK:

```bash
npm install @stellar/stellar-sdk
```

Update DonationModal usage to include poolId and contractId props:

```tsx
<DonationModal
  isOpen={isOpen}
  onClose={onClose}
  poolTitle="Pool Name"
  poolId="1"
  contractId="YOUR_CONTRACT_ID"
/>
```
