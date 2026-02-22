import React from "react";
import { PoolCard, PoolCardProps } from "./PoolCard";

// Mock Data
const MOCK_POOLS: PoolCardProps[] = [
  {
    id: "1",
    title: "Clean Water Initiative",
    description:
      "Providing clean drinking water to remote villages. Every drop counts towards a healthier future.",
    category: "Environment",
    imageUrl:
      "https://images.unsplash.com/photo-1541249591-6284fcdbf769?auto=format&fit=crop&q=80&w=800",
    goalAmount: 50000,
    raisedAmount: 32500,
    contributorsCount: 142,
    poolId: "pool_1",
    contractId: "contract_1",
  },
  {
    id: "2",
    title: "Tech Education for Girls",
    description:
      "Empowering young girls with coding skills and laptops to bridge the gender gap in tech.",
    category: "Education",
    imageUrl:
      "https://images.unsplash.com/photo-1577896851231-70ef18881754?auto=format&fit=crop&q=80&w=800",
    goalAmount: 25000,
    raisedAmount: 22000,
    contributorsCount: 89,
    poolId: "pool_2",
    contractId: "contract_2",
  },
  {
    id: "3",
    title: "Urban Reforestation",
    description:
      "Planting trees in metropolitan areas to combat pollution and lower urban temperatures.",
    category: "Environment",
    imageUrl:
      "https://images.unsplash.com/photo-1542601906990-b4d3fb778b09?auto=format&fit=crop&q=80&w=800",
    goalAmount: 15000,
    raisedAmount: 4500,
    contributorsCount: 34,
    poolId: "pool_3",
    contractId: "contract_3",
  },
  {
    id: "4",
    title: "Healthcare Clinic Access",
    description:
      "Building a mobile clinic to reach under-served populations in rural communities.",
    category: "Health",
    imageUrl:
      "https://images.unsplash.com/photo-1538108149393-fbbd81895907?auto=format&fit=crop&q=80&w=800",
    goalAmount: 100000,
    raisedAmount: 12000,
    contributorsCount: 56,
    poolId: "pool_4",
    contractId: "contract_4",
  },
  {
    id: "5",
    title: "Local Arts Center Fund",
    description:
      "Supporting local artists and providing free art workshops for children after school.",
    category: "Arts & Culture",
    imageUrl:
      "https://images.unsplash.com/photo-1460661419201-fd4cecdf8a8b?auto=format&fit=crop&q=80&w=800",
    goalAmount: 8000,
    raisedAmount: 7600,
    contributorsCount: 205,
    poolId: "pool_5",
    contractId: "contract_5",
  },
  {
    id: "6",
    title: "Disaster Relief Fund",
    description:
      "Emergency funding for immediate food, shelter, and medical supplies for victims of the recent hurricane.",
    category: "Emergency",
    imageUrl:
      "https://images.unsplash.com/photo-1588680145224-811c751270ae?auto=format&fit=crop&q=80&w=800",
    goalAmount: 200000,
    raisedAmount: 150000,
    contributorsCount: 890,
    poolId: "pool_6",
    contractId: "contract_6",
  },
];

export const PoolGrid: React.FC = () => {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
      {MOCK_POOLS.map((pool) => (
        <PoolCard key={pool.id} {...pool} />
      ))}
    </div>
  );
};
