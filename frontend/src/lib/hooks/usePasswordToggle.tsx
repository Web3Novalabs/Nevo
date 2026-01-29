"use client";
import { EyeIcon, EyeOffIcon } from "lucide-react";
import { useCallback, useMemo, useState } from "react";

export default function usePasswordToggle(initialVisible = false) {
  const [visible, setVisible] = useState(Boolean(initialVisible));

  const type = useMemo(() => (visible ? "text" : "password"), [visible]);

  const toggle = useCallback(() => setVisible((v) => !v), []);

  const icon = visible ? <EyeIcon className="size-4" /> : <EyeOffIcon className="size-4" />;
  const ariaLabel = visible ? "Hide password" : "Show password";

  return { type, visible, toggle, icon, ariaLabel };
}
