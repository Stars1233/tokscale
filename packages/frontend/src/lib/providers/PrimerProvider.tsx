"use client";

import React from "react";
import { ThemeProvider, BaseStyles } from "@primer/react";

import "@primer/primitives/dist/css/functional/themes/dark.css";

export function PrimerProvider({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider
      colorMode="night"
      nightScheme="dark"
      preventSSRMismatch
    >
      <BaseStyles>
        {children}
      </BaseStyles>
    </ThemeProvider>
  );
}

export function PrimerWrapper({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider colorMode="night" preventSSRMismatch>
      <BaseStyles>{children}</BaseStyles>
    </ThemeProvider>
  );
}
