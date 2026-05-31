/**
 * Campaign Balance Getter Edge Cases Tests — Issue #465
 */

import { describe, it, expect } from "@jest/globals";

const ERR_CAMPAIGN_NOT_FOUND = "CampaignNotFound";

interface Campaign {
  id: string;
  donations: number[];
}

const campaigns = new Map<string, Campaign>();

function get_campaign_balance(id: string): number | string {
  const campaign = campaigns.get(id);
  if (!campaign) return ERR_CAMPAIGN_NOT_FOUND;
  return campaign.donations.reduce((sum, d) => sum + d, 0);
}

function makeCampaign(id: string, donations: number[] = []): Campaign {
  const campaign: Campaign = { id, donations };
  campaigns.set(id, campaign);
  return campaign;
}

describe("get_campaign_balance edge cases", () => {
  it("new campaign returns 0 balance", () => {
    makeCampaign("camp_new");
    expect(get_campaign_balance("camp_new")).toBe(0);
  });

  it("nonexistent campaign returns CampaignNotFound error", () => {
    expect(get_campaign_balance("camp_ghost")).toBe(ERR_CAMPAIGN_NOT_FOUND);
  });

  it("campaign with donations returns correct total", () => {
    makeCampaign("camp_donations", [100, 200, 300]);
    expect(get_campaign_balance("camp_donations")).toBe(600);
  });

  it("campaign balance matches sum of all donations", () => {
    const donations = [50, 75, 125, 250];
    makeCampaign("camp_sum", donations);
    const expected = donations.reduce((a, b) => a + b, 0);
    expect(get_campaign_balance("camp_sum")).toBe(expected);
  });
});
