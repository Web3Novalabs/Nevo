// API types — generated from the server OpenAPI spec via `openapi-typescript`.
//
// The generated file lives at `lib/generated/api-schema.ts` and is produced by
// `npm run generate:api` (see package.json). Do not edit the generated file by
// hand; regenerate it whenever the server spec changes. CI runs
// `npm run check:api` to ensure the generated file is in sync.
//
// This module re-exports the generated types and provides thin, ergonomic
// aliases (ApiPool, ApiDonation, ApiUser, PaginatedResponse) derived from the
// generated schema so existing imports keep working.

import type { paths, components } from "./generated/api-schema";

export type { paths, components };

// ---------------------------------------------------------------------------
// Schema helpers
// ---------------------------------------------------------------------------

/** All named schemas exposed by the server's OpenAPI components. */
export type Schemas = components["schemas"];

/** Convenience alias for a named component schema. */
export type Schema<K extends keyof Schemas> = Schemas[K];

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/** All paths exposed by the server's OpenAPI spec. */
export type ApiPaths = paths;

/** The path item for a given route. */
export type PathItem<P extends keyof paths> = paths[P];

/**
 * Application status as stored on-chain.
 *
 * Mirrors the contract's `ApplicationStatus` enum. Using a union rather than
 * `string` ensures invalid statuses (e.g. "approved") can't be represented.
 */
export type ApplicationStatus = "Pending" | "Approved" | "Rejected";

export interface ApiTransaction {
  id: string;
  txHash: string;
  poolId: string;
  poolTitle: string;
  type: string;
  status: string;
  amount: string;
  asset: string;
  createdAt: string;
}

export interface TransactionListParams {
  page?: number;
  limit?: number;
  type?: string;
  status?: string;
  poolId?: string;
  search?: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
}

export type PaginatedTransactionsResponse = PaginatedResponse<ApiTransaction>;
/** The HTTP methods available on a given route. */
export type Method<P extends keyof paths> = keyof paths[P] & string;

/** The operation object for a given route + method. */
export type Operation<
  P extends keyof paths,
  M extends Method<P>,
> = paths[P][M];

/** The JSON request body type for a given route + method. */
export type RequestBody<
  P extends keyof paths,
  M extends Method<P>,
> = Operation<P, M> extends { requestBody: { content: { "application/json": infer B } } }
  ? B
  : never;

/** The JSON response body type for a given route + method + status. */
export type ResponseBody<
  P extends keyof paths,
  M extends Method<P>,
  S extends keyof Operation<P, M>["responses"] = 200 extends keyof Operation<P, M>["responses"]
    ? 200
    : keyof Operation<P, M>["responses"],
> = Operation<P, M>["responses"][S] extends {
  content: { "application/json": infer B };
}
  ? B
  : never;

// ---------------------------------------------------------------------------
// Thin aliases derived from the generated schema
// ---------------------------------------------------------------------------

/**
 * A pool as returned by the server.
 *
 * Derived from the generated `Pool` schema. Falls back to a structural
 * definition if the schema name differs, so imports keep working.
 */
export type ApiPool = "Pool" extends keyof Schemas
  ? Schemas["Pool"]
  : {
      id: string;
      contractPoolId: string;
      title: string;
      description: string;
      category: string;
      creatorWallet: string;
      status: string;
      goal: string;
      raised: string;
      imageUrl: string | null;
      raisedOnChain: string;
      donorCount: number;
      createdAt: string;
    };

/**
 * An application as returned by the server.
 *
 * `status` is constrained to the contract's `ApplicationStatus` union so
 * consumers can't compare against arbitrary strings.
 */
export type ApiApplication = "Application" extends keyof Schemas
  ? Omit<Schemas["Application"], "status"> & { status: ApplicationStatus }
  : {
      id: string;
      poolId: string;
      applicantWallet: string;
      status: ApplicationStatus;
      createdAt: string;
    };

/**
 * A donation as returned by the server.
 *
 * Derived from the generated `Donation` schema.
 */
export type ApiDonation = "Donation" extends keyof Schemas
  ? Schemas["Donation"]
  : {
      id: string;
      txHash: string;
      poolId: string;
      poolTitle: string;
      donorWallet: string;
      amount: string;
      asset: string;
      createdAt: string;
    };

/**
 * A user as returned by the server.
 *
 * Derived from the generated `User` schema.
 */
export type ApiUser = "User" extends keyof Schemas
  ? Schemas["User"]
  : {
      id: string;
      publicKey: string;
      displayName: string | null;
      createdAt: string;
    };

/**
 * A paginated response envelope.
 *
 * Derived from the generated `PaginatedResponse` schema when available;
 * otherwise a generic structural definition parameterised by the item type.
 */
export type PaginatedResponse<T> = "PaginatedResponse" extends keyof Schemas
  ? Omit<Schemas["PaginatedResponse"], "data"> & { data: T[] }
  : {
      data: T[];
      total: number;
      page: number;
      limit: number;
    };
