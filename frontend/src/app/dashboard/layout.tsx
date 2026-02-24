import { DashboardShell } from "@/components/dashboard/DashboardShell";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="-mt-28 min-h-screen">
      <DashboardShell>{children}</DashboardShell>
    </div>
  );
}
