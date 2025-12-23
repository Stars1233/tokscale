"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import Image from "next/image";
import { usePathname } from "next/navigation";
import styled, { keyframes } from "styled-components";
import { Avatar, ActionMenu, ActionList, Button } from "@primer/react";
import { PersonIcon, GearIcon, SignOutIcon } from "@primer/octicons-react";

interface User {
  id: string;
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
}

const pulse = keyframes`
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
`;

const Header = styled.header`
  position: sticky;
  top: 0;
  z-index: 50;
  border-bottom: 1px solid;
  backdrop-filter: blur(24px);
  display: flex;
  align-items: center;
  justify-content: center;
`;

const Container = styled.div`
  width: 100%;
  max-width: 1280px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 24px;
  padding-right: 24px;
  padding-top: 16px;
  padding-bottom: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
`;

const LogoLink = styled(Link)`
  &:hover {
    opacity: 0.8;
  }
  transition: opacity 0.15s ease-in-out;
`;

const Nav = styled.nav`
  display: none;
  align-items: center;
  gap: 4px;

  @media (min-width: 768px) {
    display: flex;
  }
`;

const NavLink = styled(Link)`
  padding: 8px 16px;
  font-size: 14px;
  font-weight: 500;
  border-radius: 8px;
  transition: background-color 0.15s ease-in-out, color 0.15s ease-in-out;
`;

const UserActions = styled.div`
  display: flex;
  align-items: center;
  gap: 12px;
`;

const LoadingSkeleton = styled.div`
  width: 36px;
  height: 36px;
  border-radius: 9999px;
  animation: ${pulse} 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
`;

const AvatarButton = styled.button`
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 9999px;
  transition: opacity 0.15s ease-in-out;

  &:hover {
    opacity: 0.8;
  }
`;

const UserInfoContainer = styled.div`
  padding-left: 12px;
  padding-right: 12px;
  padding-top: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid;
`;

const DisplayName = styled.p`
  font-size: 14px;
  font-weight: 500;
`;

const Username = styled.p`
  font-size: 12px;
`;

const GitHubIcon = styled.svg`
  width: 16px;
  height: 16px;
`;

export function Navigation() {
  const pathname = usePathname();
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    fetch("/api/auth/session")
      .then((res) => res.json())
      .then((data) => {
        setUser(data.user || null);
        setIsLoading(false);
      })
      .catch(() => {
        setIsLoading(false);
      });
  }, []);

  const isActive = (path: string) => pathname === path;

  return (
    <Header
      style={{
        width: '100%',
        borderColor: "var(--color-border-default)",
        backgroundColor: "color-mix(in srgb, var(--color-bg-default) 80%, transparent)" ,
        height: '72px',
        maxHeight: '72px',
        overflow: 'hidden'
      }}
    >
      <Container>
        <LogoLink href="/">
          <Image
            src="/tokscale-logo.svg"
            alt="tokscale"
            width={140}
            height={32}
            priority
          />
        </LogoLink>

        <Nav aria-label="Main navigation">
          <NavLink
            href="/"
            style={{
              backgroundColor: isActive("/") ? "var(--color-bg-subtle)" : "transparent",
              color: isActive("/") ? "var(--color-fg-default)" : "var(--color-fg-muted)",
            }}
          >
            Leaderboard
          </NavLink>

        </Nav>

        <UserActions>
          {isLoading ? (
            <LoadingSkeleton
              style={{
                backgroundColor: "var(--color-bg-subtle)",
                width: '36px', height: '36px', minWidth: '36px', minHeight: '36px', maxWidth: '36px', maxHeight: '36px',
              }}
            />
          ) : user ? (
            <ActionMenu>
              <ActionMenu.Anchor>
                <AvatarButton
                  aria-label={`User menu for ${user.username}`}
                  style={{ width: '36px', height: '36px', minWidth: '36px', minHeight: '36px', maxWidth: '36px', maxHeight: '36px' }}
                >
                  <Avatar
                    src={user.avatarUrl || `https://github.com/${user.username}.png`}
                    alt={user.username}
                    size={128}
                    style={{ width: '100%', height: '100%' }}
                  />
                </AvatarButton>
              </ActionMenu.Anchor>
              <ActionMenu.Overlay width="medium">
                <ActionList>
                  <ActionList.Group>
                    <UserInfoContainer
                      style={{ borderColor: "var(--color-border-default)" }}
                    >
                      <DisplayName style={{ color: "var(--color-fg-default)" }}>
                        {user.displayName || user.username}
                      </DisplayName>
                      <Username style={{ color: "var(--color-fg-muted)" }}>
                        @{user.username}
                      </Username>
                    </UserInfoContainer>
                  </ActionList.Group>
                  <ActionList.Group>
                    <ActionList.LinkItem href={`/u/${user.username}`}>
                      <ActionList.LeadingVisual>
                        <PersonIcon />
                      </ActionList.LeadingVisual>
                      Your Profile
                    </ActionList.LinkItem>
                    <ActionList.LinkItem href="/settings">
                      <ActionList.LeadingVisual>
                        <GearIcon />
                      </ActionList.LeadingVisual>
                      Settings
                    </ActionList.LinkItem>
                  </ActionList.Group>
                  <ActionList.Divider />
                  <ActionList.Group>
                    <ActionList.Item
                      variant="danger"
                      onSelect={async () => {
                        await fetch("/api/auth/logout", { method: "POST" });
                        setUser(null);
                        window.location.href = "/";
                      }}
                    >
                      <ActionList.LeadingVisual>
                        <SignOutIcon />
                      </ActionList.LeadingVisual>
                      Sign Out
                    </ActionList.Item>
                  </ActionList.Group>
                </ActionList>
              </ActionMenu.Overlay>
            </ActionMenu>
          ) : (
            <Button
              as="a"
              href="/api/auth/github"
              variant="primary"
              aria-label="Sign in with GitHub"
              leadingVisual={() => (
                <GitHubIcon fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
                </GitHubIcon>
              )}
            >
              Sign In
            </Button>
          )}
        </UserActions>
      </Container>
    </Header>
  );
}
