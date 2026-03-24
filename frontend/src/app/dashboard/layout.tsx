import { DashboardShell } from "@/components/dashboard/DashboardShell";
import { DashboardProvider } from "@/components/dashboard/DashboardContext";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="-mt-28 min-h-screen">
      <DashboardProvider>
        <DashboardShell>{children}</DashboardShell>
      </DashboardProvider>
    </div>
  );
}
