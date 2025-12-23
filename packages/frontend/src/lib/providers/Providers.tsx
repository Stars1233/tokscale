"use client";

import React from "react";
import { createGlobalStyle } from "styled-components";
import reset from "styled-reset";
import { StyledComponentsRegistry } from "./StyledComponentsRegistry";
import { PrimerProvider } from "./PrimerProvider";

const GlobalStyle = createGlobalStyle`
  ${reset}
  
  *, *::before, *::after {
    box-sizing: border-box;
  }
  
  html {
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }
  
  body {
    font-family: var(--font-figtree), "Figtree", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    line-height: 1.5;
  }
  
  code, pre, kbd, samp {
    font-family: var(--font-mono), "JetBrains Mono", "Inconsolata", monospace;
  }
  
  img, picture, video, canvas, svg {
    display: block;
    max-width: 100%;
  }
  
  input, button, textarea, select {
    font: inherit;
  }
  
  p, h1, h2, h3, h4, h5, h6 {
    overflow-wrap: break-word;
  }
  
  a {
    color: inherit;
    text-decoration: none;
  }
  
  button {
    background: none;
    border: none;
    cursor: pointer;
  }
`;

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <StyledComponentsRegistry>
      <GlobalStyle />
      <PrimerProvider>
        {children}
      </PrimerProvider>
    </StyledComponentsRegistry>
  );
}
