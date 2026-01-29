# UI Components Usage Guide

This guide provides comprehensive examples for all UI components in the design system.

---

## Table of Contents

- [Button](#button)
- [Input](#input)
- [Checkbox](#checkbox)
- [CheckboxGroup](#checkboxgroup)
- [Label](#label)
- [Toast (Sonner)](#toast-sonner)
- [Tooltip](#tooltip)

---

## Button

The Button component supports multiple variants, sizes, loading states, and icon support.

### Import

```tsx
import { Button } from "@/components/ui/button"
```

### Variants

#### Primary (Default)
```tsx
<Button>Primary Button</Button>
<Button variant="default">Default Button</Button>
```

#### Secondary
```tsx
<Button variant="secondary">Secondary Button</Button>
```

#### Destructive
```tsx
<Button variant="destructive">Delete</Button>
```

#### Outline
```tsx
<Button variant="outline">Outline Button</Button>
```

#### Ghost
```tsx
<Button variant="ghost">Ghost Button</Button>
```

#### Link
```tsx
<Button variant="link">Link Button</Button>
```

### Sizes

```tsx
<Button size="sm">Small</Button>
<Button size="default">Default</Button>
<Button size="lg">Large</Button>
```

### Icon Buttons

```tsx
import { PlusIcon, TrashIcon } from "lucide-react"

<Button size="icon">
  <PlusIcon />
</Button>

<Button size="icon-sm">
  <TrashIcon />
</Button>

<Button size="icon-lg">
  <PlusIcon />
</Button>
```

### Buttons with Icons

```tsx
import { DownloadIcon, SaveIcon } from "lucide-react"

<Button>
  <DownloadIcon />
  Download
</Button>

<Button variant="outline">
  <SaveIcon />
  Save File
</Button>
```

### Loading State

```tsx
<Button isLoading={true}>Loading...</Button>

<Button isLoading={isSubmitting} disabled={isSubmitting}>
  Submit Form
</Button>
```

### Disabled State

```tsx
<Button disabled>Disabled Button</Button>

<Button variant="destructive" disabled>
  Cannot Delete
</Button>
```

### As Child (Composition)

```tsx
<Button asChild>
  <a href="/dashboard">Go to Dashboard</a>
</Button>
```

### Complete Example

```tsx
"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { SaveIcon, Loader2Icon } from "lucide-react"

export function ButtonExample() {
  const [isSaving, setIsSaving] = useState(false)

  const handleSave = async () => {
    setIsSaving(true)
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 2000))
    setIsSaving(false)
  }

  return (
    <div className="flex gap-2">
      <Button onClick={handleSave} isLoading={isSaving}>
        <SaveIcon />
        Save Changes
      </Button>
      <Button variant="outline" disabled={isSaving}>
        Cancel
      </Button>
    </div>
  )
}
```

---

## Input

The Input component supports text, email, password (with show/hide toggle), error states, and disabled states.

### Import

```tsx
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
```

### Basic Input

```tsx
<Input type="text" placeholder="Enter your name" />
```

### Email Input

```tsx
<Input type="email" placeholder="email@example.com" />
```

### Password Input (with Toggle)

The password input automatically includes a show/hide toggle button.

```tsx
<Input type="password" placeholder="Enter password" />
```

### With Label

```tsx
<div className="space-y-2">
  <Label htmlFor="email">Email</Label>
  <Input id="email" type="email" placeholder="email@example.com" />
</div>
```

### Error State

```tsx
<Input
  type="email"
  placeholder="email@example.com"
  aria-invalid="true"
  className="border-destructive"
/>
```

### Disabled State

```tsx
<Input type="text" placeholder="Disabled input" disabled />
```

### Complete Form Example

```tsx
"use client"

import { useState } from "react"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"

export function LoginForm() {
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [errors, setErrors] = useState({ email: false, password: false })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    // Validation logic
    const newErrors = {
      email: !email.includes("@"),
      password: password.length < 8
    }
    setErrors(newErrors)
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="email">Email</Label>
        <Input
          id="email"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="email@example.com"
          aria-invalid={errors.email}
        />
        {errors.email && (
          <p className="text-sm text-destructive">Please enter a valid email</p>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="password">Password</Label>
        <Input
          id="password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Enter password"
          aria-invalid={errors.password}
        />
        {errors.password && (
          <p className="text-sm text-destructive">
            Password must be at least 8 characters
          </p>
        )}
      </div>

      <Button type="submit">Sign In</Button>
    </form>
  )
}
```

---

## Checkbox

The Checkbox component supports checked, unchecked, indeterminate states, disabled state, and error states.

### Import

```tsx
import { Checkbox } from "@/components/ui/checkbox"
import { Label } from "@/components/ui/label"
```

### Basic Checkbox

```tsx
<Checkbox />
```

### Checked State

```tsx
<Checkbox checked={true} />
<Checkbox checked={false} />
```

### Indeterminate State

```tsx
<Checkbox checked="indeterminate" />
```

### With Label

```tsx
<div className="flex items-center space-x-2">
  <Checkbox id="terms" />
  <Label htmlFor="terms">Accept terms and conditions</Label>
</div>
```

### Disabled State

```tsx
<Checkbox disabled />
<Checkbox checked={true} disabled />
```

### Error State

```tsx
<Checkbox aria-invalid="true" />
```

### Controlled Checkbox

```tsx
"use client"

import { useState } from "react"
import { Checkbox } from "@/components/ui/checkbox"
import { Label } from "@/components/ui/label"

export function ControlledCheckbox() {
  const [checked, setChecked] = useState(false)

  return (
    <div className="flex items-center space-x-2">
      <Checkbox
        id="newsletter"
        checked={checked}
        onCheckedChange={setChecked}
      />
      <Label htmlFor="newsletter">Subscribe to newsletter</Label>
    </div>
  )
}
```

### Indeterminate Example (Parent Checkbox)

```tsx
"use client"

import { useState } from "react"
import { Checkbox } from "@/components/ui/checkbox"
import { Label } from "@/components/ui/label"

export function IndeterminateExample() {
  const [items, setItems] = useState([
    { id: 1, checked: false },
    { id: 2, checked: true },
    { id: 3, checked: false }
  ])

  const allChecked = items.every(item => item.checked)
  const someChecked = items.some(item => item.checked)
  const parentState = allChecked ? true : someChecked ? "indeterminate" : false

  const handleParentChange = (checked: boolean) => {
    setItems(items.map(item => ({ ...item, checked })))
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center space-x-2">
        <Checkbox
          checked={parentState}
          onCheckedChange={handleParentChange}
        />
        <Label>Select All</Label>
      </div>
      {items.map(item => (
        <div key={item.id} className="flex items-center space-x-2 ml-6">
          <Checkbox
            checked={item.checked}
            onCheckedChange={(checked) => {
              setItems(items.map(i =>
                i.id === item.id ? { ...i, checked: checked as boolean } : i
              ))
            }}
          />
          <Label>Item {item.id}</Label>
        </div>
      ))}
    </div>
  )
}
```

---

## CheckboxGroup

The CheckboxGroup component provides a structured way to group checkboxes with labels, descriptions, and error handling.

### Import

```tsx
import { CheckboxGroup, CheckboxGroupItem } from "@/components/ui/checkbox-group"
```

### Basic Group

```tsx
<CheckboxGroup label="Preferences">
  <CheckboxGroupItem label="Email notifications" />
  <CheckboxGroupItem label="SMS notifications" />
  <CheckboxGroupItem label="Push notifications" />
</CheckboxGroup>
```

### With Description

```tsx
<CheckboxGroup
  label="Notification Settings"
  description="Choose how you want to be notified"
>
  <CheckboxGroupItem
    label="Email notifications"
    description="Receive updates via email"
  />
  <CheckboxGroupItem
    label="SMS notifications"
    description="Receive updates via SMS"
  />
</CheckboxGroup>
```

### Required Field

```tsx
<CheckboxGroup label="Terms" required>
  <CheckboxGroupItem label="I agree to the terms and conditions" />
</CheckboxGroup>
```

### Error State

```tsx
<CheckboxGroup
  label="Select at least one option"
  error="Please select at least one option"
>
  <CheckboxGroupItem label="Option 1" />
  <CheckboxGroupItem label="Option 2" />
</CheckboxGroup>
```

### Controlled Group

```tsx
"use client"

import { useState } from "react"
import { CheckboxGroup, CheckboxGroupItem } from "@/components/ui/checkbox-group"

export function ControlledGroup() {
  const [preferences, setPreferences] = useState({
    email: false,
    sms: false,
    push: false
  })

  return (
    <CheckboxGroup label="Notification Preferences">
      <CheckboxGroupItem
        label="Email notifications"
        checked={preferences.email}
        onCheckedChange={(checked) =>
          setPreferences({ ...preferences, email: checked as boolean })
        }
      />
      <CheckboxGroupItem
        label="SMS notifications"
        checked={preferences.sms}
        onCheckedChange={(checked) =>
          setPreferences({ ...preferences, sms: checked as boolean })
        }
      />
      <CheckboxGroupItem
        label="Push notifications"
        checked={preferences.push}
        onCheckedChange={(checked) =>
          setPreferences({ ...preferences, push: checked as boolean })
        }
      />
    </CheckboxGroup>
  )
}
```

### Complete Form Example

```tsx
"use client"

import { useState } from "react"
import { CheckboxGroup, CheckboxGroupItem } from "@/components/ui/checkbox-group"
import { Button } from "@/components/ui/button"

export function PreferencesForm() {
  const [preferences, setPreferences] = useState({
    email: false,
    sms: false,
    push: false
  })
  const [error, setError] = useState("")

  const handleSubmit = () => {
    if (!preferences.email && !preferences.sms && !preferences.push) {
      setError("Please select at least one notification method")
      return
    }
    setError("")
    // Submit logic
  }

  return (
    <form onSubmit={(e) => { e.preventDefault(); handleSubmit() }}>
      <CheckboxGroup
        label="Notification Preferences"
        description="Select how you want to receive notifications"
        error={error}
      >
        <CheckboxGroupItem
          label="Email notifications"
          description="Receive updates via email"
          checked={preferences.email}
          onCheckedChange={(checked) => {
            setPreferences({ ...preferences, email: checked as boolean })
            setError("")
          }}
        />
        <CheckboxGroupItem
          label="SMS notifications"
          description="Receive updates via SMS"
          checked={preferences.sms}
          onCheckedChange={(checked) => {
            setPreferences({ ...preferences, sms: checked as boolean })
            setError("")
          }}
        />
        <CheckboxGroupItem
          label="Push notifications"
          description="Receive push notifications on your device"
          checked={preferences.push}
          onCheckedChange={(checked) => {
            setPreferences({ ...preferences, push: checked as boolean })
            setError("")
          }}
        />
      </CheckboxGroup>
      <Button type="submit" className="mt-4">Save Preferences</Button>
    </form>
  )
}
```

---

## Label

The Label component is used to associate labels with form inputs for better accessibility.

### Import

```tsx
import { Label } from "@/components/ui/label"
```

### Basic Usage

```tsx
<Label htmlFor="email">Email Address</Label>
<Input id="email" type="email" />
```

### With Checkbox

```tsx
<div className="flex items-center space-x-2">
  <Checkbox id="terms" />
  <Label htmlFor="terms">I agree to the terms</Label>
</div>
```

### Required Field Indicator

```tsx
<Label htmlFor="name" className="after:content-['*'] after:ml-0.5 after:text-destructive">
  Full Name
</Label>
```

---

## Toast (Sonner)

The Toast component provides success, error, warning, and info notifications with auto-dismiss and close button functionality.

### Setup

First, add the Toaster to your root layout:

```tsx
// app/layout.tsx
import { Toaster } from "@/components/ui/sonner"

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        {children}
        <Toaster />
      </body>
    </html>
  )
}
```

### Import

```tsx
import { toast } from "sonner"
```

### Success Toast

```tsx
import { toast } from "sonner"

toast.success("Operation completed successfully!")
```

### Error Toast

```tsx
toast.error("Something went wrong!")
```

### Warning Toast

```tsx
toast.warning("Please review your input")
```

### Info Toast

```tsx
toast.info("New update available")
```

### Loading Toast

```tsx
const toastId = toast.loading("Processing...")

// Later, update or dismiss
toast.success("Done!", { id: toastId })
// or
toast.dismiss(toastId)
```

### Custom Duration

```tsx
// Default Toaster with custom duration (10 seconds)
<Toaster duration={10000} />

// Per-toast duration
toast.success("Saved!", { duration: 2000 })
```

### Without Close Button

```tsx
<Toaster closeButton={false} />
```

### Custom Toast

```tsx
toast("Custom message", {
  description: "This is a description",
  action: {
    label: "Undo",
    onClick: () => console.log("Undo clicked")
  }
})
```

### Promise Toast

```tsx
const promise = fetch("/api/data")

toast.promise(promise, {
  loading: "Loading...",
  success: "Data loaded successfully!",
  error: "Failed to load data"
})
```

### Complete Example

```tsx
"use client"

import { toast } from "sonner"
import { Button } from "@/components/ui/button"

export function ToastExamples() {
  const handleSuccess = () => {
    toast.success("Operation completed successfully!")
  }

  const handleError = () => {
    toast.error("An error occurred!")
  }

  const handlePromise = async () => {
    const promise = new Promise((resolve) => {
      setTimeout(resolve, 2000)
    })

    toast.promise(promise, {
      loading: "Processing...",
      success: "Completed!",
      error: "Failed!"
    })
  }

  return (
    <div className="flex gap-2">
      <Button onClick={handleSuccess}>Success Toast</Button>
      <Button onClick={handleError} variant="destructive">
        Error Toast
      </Button>
      <Button onClick={handlePromise}>Promise Toast</Button>
    </div>
  )
}
```

### Toaster Configuration

```tsx
// Custom duration and close button settings
<Toaster
  duration={5000}        // Auto-dismiss after 5 seconds
  closeButton={true}     // Show close button (default)
  position="top-right"   // Position on screen
/>
```

### Available Toaster Props

- `duration` (number, default: 4000) - Auto-dismiss duration in milliseconds
- `closeButton` (boolean, default: true) - Show/hide close button
- `position` - Toast position: "top-left" | "top-center" | "top-right" | "bottom-left" | "bottom-center" | "bottom-right"

---

## Tooltip

The Tooltip component provides contextual information on hover or focus.

### Import

```tsx
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
  TooltipProvider
} from "@/components/ui/tooltip"
```

### Basic Tooltip

```tsx
<Tooltip>
  <TooltipTrigger>
    <Button>Hover me</Button>
  </TooltipTrigger>
  <TooltipContent>
    <p>This is a tooltip</p>
  </TooltipContent>
</Tooltip>
```

### Different Positions

```tsx
<Tooltip>
  <TooltipTrigger>
    <Button>Top</Button>
  </TooltipTrigger>
  <TooltipContent side="top">
    <p>Tooltip on top</p>
  </TooltipContent>
</Tooltip>

<Tooltip>
  <TooltipTrigger>
    <Button>Bottom</Button>
  </TooltipTrigger>
  <TooltipContent side="bottom">
    <p>Tooltip on bottom</p>
  </TooltipContent>
</Tooltip>

<Tooltip>
  <TooltipTrigger>
    <Button>Left</Button>
  </TooltipTrigger>
  <TooltipContent side="left">
    <p>Tooltip on left</p>
  </TooltipContent>
</Tooltip>

<Tooltip>
  <TooltipTrigger>
    <Button>Right</Button>
  </TooltipTrigger>
  <TooltipContent side="right">
    <p>Tooltip on right</p>
  </TooltipContent>
</Tooltip>
```

### With Icon Button

```tsx
import { InfoIcon } from "lucide-react"

<Tooltip>
  <TooltipTrigger asChild>
    <Button size="icon" variant="ghost">
      <InfoIcon />
    </Button>
  </TooltipTrigger>
  <TooltipContent>
    <p>More information</p>
  </TooltipContent>
</Tooltip>
```

### Custom Delay

```tsx
<TooltipProvider delayDuration={300}>
  <Tooltip>
    <TooltipTrigger>
      <Button>Delayed tooltip</Button>
    </TooltipTrigger>
    <TooltipContent>
      <p>This tooltip has a 300ms delay</p>
    </TooltipContent>
  </Tooltip>
</TooltipProvider>
```

### Multiple Tooltips

```tsx
<TooltipProvider>
  <div className="flex gap-4">
    <Tooltip>
      <TooltipTrigger>
        <Button>Button 1</Button>
      </TooltipTrigger>
      <TooltipContent>
        <p>First tooltip</p>
      </TooltipContent>
    </Tooltip>

    <Tooltip>
      <TooltipTrigger>
        <Button>Button 2</Button>
      </TooltipTrigger>
      <TooltipContent>
        <p>Second tooltip</p>
      </TooltipContent>
    </Tooltip>
  </div>
</TooltipProvider>
```

### Complete Example

```tsx
"use client"

import {
  Tooltip,
  TooltipTrigger,
  TooltipContent
} from "@/components/ui/tooltip"
import { Button } from "@/components/ui/button"
import { HelpCircleIcon, SettingsIcon } from "lucide-react"

export function TooltipExample() {
  return (
    <div className="flex gap-4 items-center">
      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="outline">
            <HelpCircleIcon />
            Help
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <p>Click for help and support</p>
        </TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button size="icon" variant="ghost">
            <SettingsIcon />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom">
          <p>Settings</p>
        </TooltipContent>
      </Tooltip>
    </div>
  )
}
```

---

## Best Practices

### Accessibility

- Always use `Label` components with form inputs
- Provide `aria-invalid` for error states
- Use semantic HTML and proper ARIA attributes
- Ensure keyboard navigation works for all interactive elements

### Form Handling

- Use controlled components for form state management
- Validate inputs and show error messages appropriately
- Disable submit buttons during form submission
- Provide loading states for async operations

### Toast Notifications

- Use appropriate toast types (success, error, warning, info)
- Keep messages concise and actionable
- Use promises for async operations
- Consider user experience when setting durations

### Component Composition

- Use `asChild` prop when you need to compose components
- Leverage component variants for different use cases
- Keep components focused and reusable

---

## TypeScript Support

All components are fully typed with TypeScript. Import types when needed:

```tsx
import type { ToasterPropsExtended } from "@/components/ui/sonner"
```

---

## Additional Resources

- [shadcn/ui Documentation](https://ui.shadcn.com)
- [Radix UI Documentation](https://www.radix-ui.com)
- [Sonner Documentation](https://sonner.emilkowal.ski)
