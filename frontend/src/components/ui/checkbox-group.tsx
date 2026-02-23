"use client"

import * as React from "react"
import { cn } from "@/lib/utils"
import { Label } from "./label"
import { Checkbox } from "./checkbox"

interface CheckboxGroupProps {
  label?: string
  error?: string
  description?: string
  className?: string
  children: React.ReactNode
  required?: boolean
}

function CheckboxGroup({
  label,
  error,
  description,
  className,
  children,
  required,
  ...props
}: CheckboxGroupProps & React.HTMLAttributes<HTMLDivElement>) {
  const groupId = React.useId()
  const errorId = error ? `${groupId}-error` : undefined
  const descriptionId = description ? `${groupId}-description` : undefined

  return (
    <div
      data-slot="checkbox-group"
      className={cn("space-y-2", className)}
      {...props}
    >
      {label && (
        <Label
          htmlFor={groupId}
          className={cn(
            "text-sm font-medium",
            error && "text-destructive",
            required && "after:content-['*'] after:ml-0.5 after:text-destructive"
          )}
        >
          {label}
        </Label>
      )}
      {description && (
        <p
          id={descriptionId}
          className="text-sm text-muted-foreground"
        >
          {description}
        </p>
      )}
      <div
        role="group"
        aria-labelledby={label ? groupId : undefined}
        aria-describedby={cn(descriptionId, errorId)}
        aria-invalid={error ? "true" : undefined}
        className="space-y-2"
      >
        {children}
      </div>
      {error && (
        <p
          id={errorId}
          role="alert"
          className="text-sm text-destructive"
          aria-live="polite"
        >
          {error}
        </p>
      )}
    </div>
  )
}

interface CheckboxGroupItemProps {
  label: string
  error?: boolean
  description?: string
  className?: string
}

function CheckboxGroupItem({
  label,
  error,
  description,
  className,
  children,
  ...props
}: CheckboxGroupItemProps &
  React.ComponentProps<typeof Checkbox> & {
    children?: React.ReactNode
  }) {
  const itemId = React.useId()

  return (
    <div
      data-slot="checkbox-group-item"
      className={cn("flex items-start space-x-2", className)}
    >
      <Checkbox
        id={itemId}
        // aria-invalid={error}
        aria-describedby={description ? `${itemId}-description` : undefined}
        {...props}
      />
      <div className="grid gap-1.5 leading-none">
        <Label
          htmlFor={itemId}
          className={cn(
            "text-sm font-normal cursor-pointer peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
            error && "text-destructive"
          )}
        >
          {label}
        </Label>
        {description && (
          <p
            id={`${itemId}-description`}
            className="text-sm text-muted-foreground"
          >
            {description}
          </p>
        )}
      </div>
      {children}
    </div>
  )
}

export { CheckboxGroup, CheckboxGroupItem }
