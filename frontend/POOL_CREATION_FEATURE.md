# Pool Creation Stepper UI

## Overview
A beautiful 3-step wizard for creating new donation pools on the Stellar blockchain.

## Features

### Step 1: Basic Information
- Pool name (required)
- Category selection (Education, Medical, Community, Environment, Arts, Other)
- Description with character counter (required)
- Optional end date picker

### Step 2: Financial Details
- Funding goal in XLM (required)
- Minimum contribution amount (optional)
- Beneficiary wallet address (required)
- Visibility toggle (Public/Private)

### Step 3: Review & Submit
- Summary of all entered information
- Organized display by step
- Confirmation notice about blockchain immutability
- Submit button with loading state

## UI/UX Highlights

### Progress Indicators
- Animated progress bar showing completion percentage
- Step circles with checkmarks for completed steps
- Active step highlighting with emerald/cyan gradient
- Step labels with smooth color transitions

### Smooth Transitions
- 220ms fade and slide animations between steps
- Direction-aware animations (forward/backward)
- Progress bar transitions with glow effects

### Navigation
- "Back" button (disabled on first step)
- "Next" button with validation
- "Create Pool" button on final step
- Active state animations and hover effects

### Validation
- Real-time form validation
- Visual feedback with disabled button states
- Required field indicators
- Input focus states

## Technical Implementation

### Components
- `CreatePoolStepper.tsx` - Main stepper component
- `BasicInfoStep.tsx` - Step 1 form
- `FinancialsStep.tsx` - Step 2 form
- `ReviewStep.tsx` - Step 3 summary

### Tech Stack
- Next.js 15 with React 19
- TypeScript for type safety
- Tailwind CSS for styling
- Framer Motion for animations

### Form State Management
```typescript
interface FormData {
  // Step 1
  poolName: string;
  category: string;
  description: string;
  endDate: string;
  // Step 2
  fundingGoal: string;
  minContribution: string;
  beneficiaryWallet: string;
  visibility: "Public" | "Private";
}
```

## Usage

Navigate to `/dashboard/pools/create` to access the pool creation wizard.

## CI/CD

The feature is covered by the Frontend CI workflow which:
- Runs ESLint for code quality checks
- Verifies the Next.js build succeeds
- Triggers on changes to the `frontend/**` directory

## Future Enhancements

Potential improvements:
- Image upload for pool thumbnail
- Rich text editor for description
- Multi-currency support
- Draft saving functionality
- Pool templates
