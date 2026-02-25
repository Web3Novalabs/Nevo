"use client";

import React, { createContext, useContext, useState, useEffect } from "react";

export interface StatData {
  title: string;
  value: string;
  iconName: string;
  trend?: string;
}

export interface ActivityData {
  id: string;
  type: "donation" | "pool_created" | "reward";
  user: string;
  amount?: string;
  poolName?: string;
  timestamp: string;
}

interface DashboardContextType {
  stats: StatData[];
  activities: ActivityData[];
  isLoading: boolean;
}

const DashboardContext = createContext<DashboardContextType | undefined>(undefined);

export function DashboardProvider({ children }: { children: React.ReactNode }) {
  const [stats, setStats] = useState<StatData[]>([]);
  const [activities, setActivities] = useState<ActivityData[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Simulate network delay for fetching dashboard data
    const timer = setTimeout(() => {
      setStats([
        { title: "Total Donated", value: "$45,231.89", iconName: "Banknotes", trend: "+12.5%" },
        { title: "Active Pools", value: "24", iconName: "Droplets", trend: "+2" },
        { title: "Impact Score", value: "89.2", iconName: "Activity", trend: "+4.1" },
      ]);
      setActivities([
        { id: "1", type: "donation", user: "Alice", amount: "$500", poolName: "Ocean Cleanup", timestamp: "10 mins ago" },
        { id: "2", type: "pool_created", user: "Bob", poolName: "Reforestation Initiative", timestamp: "2 hours ago" },
        { id: "3", type: "reward", user: "Charlie", amount: "50 NEVO", timestamp: "5 hours ago" },
        { id: "4", type: "donation", user: "Diana", amount: "$150", poolName: "Local Shelter", timestamp: "1 day ago" },
      ]);
      setIsLoading(false);
    }, 1500);

    return () => clearTimeout(timer);
  }, []);

  return (
    <DashboardContext.Provider value={{ stats, activities, isLoading }}>
      {children}
    </DashboardContext.Provider>
  );
}

export function useDashboard() {
  const context = useContext(DashboardContext);
  if (context === undefined) {
    throw new Error("useDashboard must be used within a DashboardProvider");
  }
  return context;
}
