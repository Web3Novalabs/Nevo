"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Checkbox } from "@/components/ui/checkbox"
import {
  CheckboxGroup,
  CheckboxGroupItem,
} from "@/components/ui/checkbox-group"
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip"
import { toast } from "sonner"
import {
  PlusIcon,
  TrashIcon,
  DownloadIcon,
  SaveIcon,
  SunIcon,
  MoonIcon,
  InfoIcon,
} from "lucide-react"

const paletteTokens = [
  { name: "Background", var: "background", textVar: "foreground" },
  { name: "Foreground", var: "foreground", textVar: "background" },
  { name: "Primary", var: "primary", textVar: "primary-foreground" },
  { name: "Secondary", var: "secondary", textVar: "secondary-foreground" },
  { name: "Accent", var: "accent", textVar: "accent-foreground" },
  { name: "Muted", var: "muted", textVar: "muted-foreground" },
  { name: "Destructive", var: "destructive", textVar: "primary-foreground" },
  { name: "Border", var: "border", textVar: "foreground" },
  { name: "Ring", var: "ring", textVar: "background" },
  { name: "Card", var: "card", textVar: "card-foreground" },
  { name: "Popover", var: "popover", textVar: "popover-foreground" },
  { name: "Input", var: "input", textVar: "foreground" },
] as const

function PaletteSwatch({
  name,
  var: cssVar,
  textVar,
}: {
  name: string
  var: string
  textVar: string
}) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-xs font-medium text-muted-foreground">{name}</span>
      <div
        className="h-14 rounded-lg border border-border shadow-sm"
        style={{
          backgroundColor: `var(--${cssVar})`,
          color: `var(--${textVar})`,
        }}
      >
        <span className="flex h-full items-center justify-center text-sm font-medium">
          Aa
        </span>
      </div>
    </div>
  )
}

export default function UIDemoPage() {
  const [isDark, setIsDark] = useState(false)

  return (
    <>
      <div
        className={`min-h-screen rounded-xl border border-border p-8 transition-colors ${isDark ? "dark" : ""}`}
        style={{
          backgroundColor: "var(--background)",
          color: "var(--foreground)",
        }}
      >
        <div className="mx-auto max-w-4xl space-y-12">
          {/* Header + theme toggle */}
          <div className="flex flex-wrap items-center justify-between gap-4">
            <h1 className="text-3xl font-bold text-foreground">
              UI Components & Palette
            </h1>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setIsDark((d) => !d)}
              aria-label={isDark ? "Switch to light mode" : "Switch to dark mode"}
            >
              {isDark ? (
                <SunIcon className="size-4" />
              ) : (
                <MoonIcon className="size-4" />
              )}
            </Button>
          </div>

          {/* Palette section */}
          <section className="space-y-4">
            <h2 className="text-xl font-semibold text-foreground">
              Palette (all variants)
            </h2>
            <p className="text-sm text-muted-foreground">
              Design tokens used across components. Toggle dark mode to see both
              themes.
            </p>
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4">
              {paletteTokens.map((token) => (
                <PaletteSwatch
                  key={token.var}
                  name={token.name}
                  var={token.var}
                  textVar={token.textVar}
                />
              ))}
            </div>
          </section>

          {/* Buttons */}
          <section className="space-y-4">
            <h2 className="text-xl font-semibold text-foreground">Button</h2>
            <p className="text-sm text-muted-foreground">
              Variants: default (primary), secondary, destructive, outline,
              ghost, link.
            </p>
            <div className="flex flex-wrap gap-2">
              <Button>Default</Button>
              <Button variant="secondary">Secondary</Button>
              <Button variant="destructive">Destructive</Button>
              <Button variant="outline">Outline</Button>
              <Button variant="ghost">Ghost</Button>
              <Button variant="link">Link</Button>
            </div>
            <p className="text-sm text-muted-foreground">Sizes</p>
            <div className="flex flex-wrap items-center gap-2">
              <Button size="sm">Small</Button>
              <Button size="default">Default</Button>
              <Button size="lg">Large</Button>
              <Button size="icon">
                <PlusIcon />
              </Button>
              <Button size="icon-sm">
                <TrashIcon />
              </Button>
              <Button size="icon-lg">
                <PlusIcon />
              </Button>
            </div>
            <p className="text-sm text-muted-foreground">With icons</p>
            <div className="flex flex-wrap gap-2">
              <Button>
                <DownloadIcon />
                Download
              </Button>
              <Button variant="outline">
                <SaveIcon />
                Save
              </Button>
            </div>
            <p className="text-sm text-muted-foreground">States</p>
            <div className="flex flex-wrap gap-2">
              <Button disabled>Disabled</Button>
              <Button isLoading>Loading</Button>
            </div>
          </section>

          {/* Input */}
          <section className="space-y-4">
            <h2 className="text-xl font-semibold text-foreground">Input</h2>
            <p className="text-sm text-muted-foreground">
              Text, email, password (with show/hide), with label.
            </p>
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="demo-name">Name</Label>
                <Input id="demo-name" type="text" placeholder="Your name" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="demo-email">Email</Label>
                <Input
                  id="demo-email"
                  type="email"
                  placeholder="email@example.com"
                />
              </div>
              <div className="space-y-2 sm:col-span-2">
                <Label htmlFor="demo-password">Password</Label>
                <Input
                  id="demo-password"
                  type="password"
                  placeholder="Enter password"
                />
              </div>
            </div>
          </section>

          {/* Checkbox & CheckboxGroup */}
          <section className="space-y-4">
            <h2 className="text-xl font-semibold text-foreground">
              Checkbox & CheckboxGroup
            </h2>
            <p className="text-sm text-muted-foreground">
              Single checkbox and group with label/description/error.
            </p>
            <div className="flex flex-wrap gap-6">
              <div className="flex items-center gap-2">
                <Checkbox id="single" />
                <Label htmlFor="single">Single option</Label>
              </div>
              <CheckboxGroup label="Options" description="Pick one or more.">
                <CheckboxGroupItem id="opt-a" label="Option A" />
                <CheckboxGroupItem id="opt-b" label="Option B" />
                <CheckboxGroupItem id="opt-c" label="Option C" />
              </CheckboxGroup>
              <CheckboxGroup
                label="Required"
                description="With error state."
                error="At least one required"
                required
              >
                <CheckboxGroupItem id="req-1" label="Required 1" />
                <CheckboxGroupItem id="req-2" label="Required 2" error />
              </CheckboxGroup>
            </div>
          </section>

          {/* Tooltip */}
          <section className="space-y-4">
            <h2 className="text-xl font-semibold text-foreground">Tooltip</h2>
            <p className="text-sm text-muted-foreground">
              Hover to see tooltip (uses foreground/background).
            </p>
            <div className="flex flex-wrap gap-2">
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="outline" size="icon">
                    <InfoIcon />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Help tooltip</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="secondary">Hover me</Button>
                </TooltipTrigger>
                <TooltipContent>Secondary button with tooltip</TooltipContent>
              </Tooltip>
            </div>
          </section>

          {/* Toast (Sonner) */}
          <section className="space-y-4">
            <h2 className="text-xl font-semibold text-foreground">
              Toast (Sonner)
            </h2>
            <p className="text-sm text-muted-foreground">
              Triggers use primary/destructive and toast styling.
            </p>
            <div className="flex flex-wrap gap-2">
              <Button
                onClick={() => toast.success("Success message")}
              >
                Success
              </Button>
              <Button
                variant="outline"
                onClick={() => toast.info("Info message")}
              >
                Info
              </Button>
              <Button
                variant="secondary"
                onClick={() => toast.warning("Warning message")}
              >
                Warning
              </Button>
              <Button
                variant="destructive"
                onClick={() => toast.error("Error message")}
              >
                Error
              </Button>
              <Button
                variant="ghost"
                onClick={() => toast.loading("Loading...")}
              >
                Loading
              </Button>
            </div>
          </section>
        </div>
      </div>
    </>
  )
}
