"use client";

import React, { useState } from "react";
import { TrendingUp, Users } from "lucide-react";
import Image from "next/image";
import { DonationModal } from "./DonationModal";
import { Button } from "./ui/button";

export interface PoolCardProps {
  id: string;
  title: string;
  description: string;
  category: string;
  imageUrl: string;
  goalAmount: number;
  raisedAmount: number;
  contributorsCount: number;
}

export const PoolCard: React.FC<PoolCardProps> = ({
  title,
  description,
  category,
  imageUrl,
  goalAmount,
  raisedAmount,
  contributorsCount,
}) => {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const progressPercent = Math.min((raisedAmount / goalAmount) * 100, 100);

  return (
    <>
      <div className="group bg-slate-50 dark:bg-slate-800/50 rounded-2xl border border-slate-200 dark:border-slate-700 overflow-hidden hover:shadow-2xl transition-all duration-300 hover:-translate-y-1 cursor-pointer flex flex-col h-full">
        {/* Image Container with Hover Scale Result */}
        <div className="relative h-48 w-full overflow-hidden">
          <Image
            src={imageUrl}
            alt={title}
            width={800}
            height={400}
            className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
          />
          {/* Category Badge over image */}
          <div className="absolute top-4 left-4">
            <span className="px-3 py-1 text-xs font-semibold rounded-full bg-[#0F172A]/80 text-white backdrop-blur-sm border border-slate-700">
              {category}
            </span>
          </div>
        </div>

        {/* Card Content */}
        <div className="p-6 flex flex-col flex-grow">
          <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2 line-clamp-1 group-hover:text-blue-500 dark:group-hover:text-cyan-400 transition-colors">
            {title}
          </h3>
          <p className="text-slate-600 dark:text-slate-400 text-sm mb-6 line-clamp-2 flex-grow">
            {description}
          </p>

          {/* Progress Section */}
          <div className="mt-auto space-y-4">
            <div className="flex justify-between items-end text-sm">
              <div>
                <p className="text-slate-500 dark:text-slate-400">Raised</p>
                <p className="font-bold text-slate-900 dark:text-white text-lg">
                  ${raisedAmount.toLocaleString()}
                </p>
              </div>
              <div className="text-right">
                <p className="text-slate-500 dark:text-slate-400">Goal</p>
                <p className="font-semibold text-slate-700 dark:text-slate-300">
                  ${goalAmount.toLocaleString()}
                </p>
              </div>
            </div>

            {/* Progress Bar Container */}
            <div className="w-full bg-slate-200 dark:bg-slate-700 rounded-full h-2.5 overflow-hidden">
              <div
                className="bg-gradient-to-r from-blue-500 to-cyan-400 h-2.5 rounded-full transition-all duration-1000 ease-out"
                style={{ width: `${progressPercent}%` }}
              />
            </div>

            {/* Footer Stats & Action */}
            <div className="pt-4 space-y-4">
              <div className="flex justify-between items-center text-xs text-slate-500 dark:text-slate-400 pt-2 border-t border-slate-200 dark:border-slate-700">
                <div className="flex items-center space-x-1">
                  <Users size={14} />
                  <span>{contributorsCount} Contributors</span>
                </div>
                <div className="flex items-center space-x-1 text-blue-600 dark:text-cyan-400">
                  <TrendingUp size={14} />
                  <span className="font-semibold">
                    {Math.round(progressPercent)}%
                  </span>
                </div>
              </div>

              {/* Donate Button */}
              <Button
                onClick={() => setIsModalOpen(true)}
                className="w-full bg-slate-900 hover:bg-slate-800 dark:bg-white dark:hover:bg-slate-200 text-white dark:text-slate-900 transition-colors"
              >
                Donate Now
              </Button>
            </div>
          </div>
        </div>
      </div>

      <DonationModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        poolTitle={title}
      />
    </>
  );
};
