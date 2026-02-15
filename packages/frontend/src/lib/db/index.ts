import { drizzle } from "drizzle-orm/postgres-js";
import * as schema from "./schema";

const connectionString = process.env.DATABASE_URL;

if (!connectionString) {
  throw new Error("DATABASE_URL environment variable is not set");
}

// Use drizzle's config-based API to create the postgres client internally.
// Passing a `postgres` Sql instance directly causes type errors on Vercel
// due to duplicate package resolution in the monorepo (two copies of postgres
// with incompatible branded types).
export const db = drizzle({
  connection: {
    url: connectionString,
    ssl: process.env.NODE_ENV === "production" ? "require" : false,
    max: 1, // Serverless-friendly connection limit
  },
  schema,
});

export * from "./schema";
