"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import Image from "next/image";
import { usePathname } from "next/navigation";
import styled, { css, keyframes } from "styled-components";
import { Avatar, ActionMenu, ActionList } from "@primer/react";
import { PersonIcon, GearIcon, SignOutIcon } from "@primer/octicons-react";

interface User {
  id: string;
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
}

const MOBILE_BREAKPOINT = "520px";

const pulse = keyframes`
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
`;

const NavContainer = styled.nav`
  position: fixed;
  top: 17px;
  left: 50%;
  transform: translateX(-50%);
  height: 40px;
  background: rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(10px);
  border-radius: 32px;
  padding: 4px;
  display: flex;
  align-items: center;
  gap: 4px;
  z-index: 50;
  
  @media (max-width: 767px) {
    max-width: 100%;
    gap: 2px;
  }
`;

const NavItemBase = styled.a<{ $isActive: boolean }>`
  font-family: 'Figtree', sans-serif;
  font-weight: 700;
  font-size: 16px;
  text-transform: uppercase;
  padding: 8px 23px;
  border-radius: 1000px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-decoration: none;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  min-width: 0;
  
  @media (max-width: 767px) {
    padding: 8px 10px;
  }
  
  ${({ $isActive }) =>
    $isActive
      ? css`
    background: rgba(235, 242, 245, 0.96);
    border: 1px solid rgba(235, 242, 245, 0.96);
    color: #000000;
  `
      : css`
    background: transparent;
    border: 1px solid transparent;
    color: #D9D9D9;
    
    &:hover {
      color: #ffffff;
    }
  `}
`;

const NavItemLink = styled(Link)<{ $isActive: boolean }>`
  font-family: 'Figtree', sans-serif;
  font-weight: 700;
  font-size: 16px;
  text-transform: uppercase;
  padding: 8px 23px;
  border-radius: 1000px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-decoration: none;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  min-width: 0;
  
  @media (max-width: 767px) {
    padding: 8px 10px;
  }
  
  ${({ $isActive }) =>
    $isActive
      ? css`
    background: rgba(235, 242, 245, 0.96);
    border: 1px solid rgba(235, 242, 245, 0.96);
    color: #000000;
  `
      : css`
    background: transparent;
    border: 1px solid transparent;
    color: #D9D9D9;
    
    &:hover {
      color: #ffffff;
    }
  `}
`;

const LoadingSkeleton = styled.div`
  width: 32px;
  height: 32px;
  border-radius: 1000px;
  animation: ${pulse} 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  background-color: rgba(255, 255, 255, 0.1);
  flex-shrink: 0;
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

const SignInButton = styled.a`
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: center;
  padding: 6px 10px 6px 6px;
  gap: 8px;
  height: 32px;
  background: #0073FF;
  border-radius: 1000px;
  text-decoration: none;
  cursor: pointer;
  transition: opacity 0.15s ease;
  flex-shrink: 0;
  
  &:hover {
    opacity: 0.9;
  }
  
  @media (max-width: 374px) {
    padding: 6px;
    gap: 4px;
  }
`;

const SignInIcon = styled.div`
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
`;

const SignInText = styled.span`
  font-family: var(--font-figtree), 'Figtree', sans-serif;
  font-weight: 700;
  font-size: 14px;
  line-height: 94%;
  text-align: center;
  letter-spacing: -0.05em;
  color: #FFFFFF;
  white-space: nowrap;
`;

const SignInTextFull = styled(SignInText)`
  @media (max-width: 767px) {
    display: none;
  }
`;

const SignInTextCompact = styled(SignInText)`
  @media (min-width: 768px) {
    display: none;
  }
`;

const ProfileButton = styled.button`
  box-sizing: border-box;
  display: flex;
  justify-content: center;
  align-items: center;
  width: 32px;
  height: 32px;
  background: #0073FF;
  border: 1px solid #0073FF;
  border-radius: 1000px;
  padding: 0;
  cursor: pointer;
  overflow: hidden;
  transition: opacity 0.15s ease;
  flex-shrink: 0;
  
  &:hover {
    opacity: 0.9;
  }
`;

const DesktopNavItems = styled.div`
  display: contents;

  @media (max-width: ${MOBILE_BREAKPOINT}) {
    display: none;
  }
`;

const DesktopAuthSection = styled.div`
  display: contents;

  @media (max-width: ${MOBILE_BREAKPOINT}) {
    display: none;
  }
`;

const HamburgerButton = styled.button`
  display: none;
  width: 32px;
  height: 32px;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #D9D9D9;
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
  border-radius: 1000px;
  transition: color 0.15s ease;

  &:hover {
    color: #ffffff;
  }

  @media (max-width: ${MOBILE_BREAKPOINT}) {
    display: flex;
  }
`;

const MobileMenuOverlay = styled.div<{ $isOpen: boolean }>`
  position: fixed;
  inset: 0;
  z-index: 999;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  opacity: ${({ $isOpen }) => ($isOpen ? 1 : 0)};
  pointer-events: ${({ $isOpen }) => ($isOpen ? "auto" : "none")};
  transition: opacity 0.2s ease;
`;

const MobileMenuPanel = styled.div<{ $isOpen: boolean }>`
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: 100%;

  z-index: 1000;
  background: #0D1117;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  transform: ${({ $isOpen }) => ($isOpen ? "translateX(0)" : "translateX(100%)")};
  transition: transform 0.25s ease;
`;

const MobileMenuHeader = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
`;

const MobileMenuLogoWrapper = styled.div`
  position: relative;
  width: 140px;
  height: 30px;
  flex-shrink: 0;
`;

const MobileMenuLogoImage = styled(Image)`
  object-fit: contain;
`;

const MobileCloseButton = styled.button`
  display: flex;
  width: 32px;
  height: 32px;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #D9D9D9;
  cursor: pointer;
  padding: 0;
  border-radius: 1000px;
  transition: color 0.15s ease;

  &:hover {
    color: #ffffff;
  }
`;

const MobileNavLinks = styled.div`
  flex: 1;
  display: flex;
  flex-direction: column;
  padding-top: 8px;
`;

const MobileNavLinkItem = styled(Link)<{ $isActive: boolean }>`
  font-family: var(--font-figtree), 'Figtree', sans-serif;
  font-weight: 700;
  font-size: 18px;
  text-transform: uppercase;
  color: ${({ $isActive }) => ($isActive ? "#ffffff" : "#D9D9D9")};
  background: ${({ $isActive }) => ($isActive ? "rgba(0, 115, 255, 0.1)" : "transparent")};
  text-decoration: none;
  padding: 16px 20px;
  display: block;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  transition: all 0.15s ease;

  &:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.04);
  }
`;

const MobileNavLinkExternal = styled.a<{ $isActive: boolean }>`
  font-family: var(--font-figtree), 'Figtree', sans-serif;
  font-weight: 700;
  font-size: 18px;
  text-transform: uppercase;
  color: ${({ $isActive }) => ($isActive ? "#ffffff" : "#D9D9D9")};
  text-decoration: none;
  padding: 16px 20px;
  display: block;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  transition: all 0.15s ease;

  &:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.04);
  }
`;

const MobileAuthSection = styled.div`
  padding: 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  margin-top: auto;
`;

const MobileSignInButton = styled.a`
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 44px;
  background: #0073FF;
  border-radius: 12px;
  text-decoration: none;
  cursor: pointer;
  transition: opacity 0.15s ease;

  &:hover {
    opacity: 0.9;
  }
`;

const MobileSignInText = styled.span`
  font-family: var(--font-figtree), 'Figtree', sans-serif;
  font-weight: 700;
  font-size: 15px;
  color: #ffffff;
`;

const MobileUserCard = styled.div`
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
`;

const MobileUserDetails = styled.div`
  flex: 1;
  min-width: 0;
`;

const MobileDisplayName = styled.p`
  font-family: var(--font-figtree), 'Figtree', sans-serif;
  font-weight: 600;
  font-size: 15px;
  color: #ffffff;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
`;

const MobileUsername = styled.p`
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
`;

const MobileUserActions = styled.div`
  display: flex;
  flex-direction: column;
  gap: 4px;
`;

const MobileUserAction = styled(Link)`
  font-family: var(--font-figtree), 'Figtree', sans-serif;
  font-weight: 500;
  font-size: 14px;
  color: #D9D9D9;
  text-decoration: none;
  padding: 10px 12px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 10px;
  transition: all 0.15s ease;

  &:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.06);
  }

  svg {
    color: rgba(255, 255, 255, 0.4);
    flex-shrink: 0;
  }
`;

const MobileSignOutButton = styled.button`
  font-family: var(--font-figtree), 'Figtree', sans-serif;
  font-weight: 500;
  font-size: 14px;
  color: #F85149;
  background: transparent;
  border: none;
  padding: 10px 12px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  cursor: pointer;
  transition: all 0.15s ease;

  &:hover {
    background: rgba(248, 81, 73, 0.1);
  }

  svg {
    color: #F85149;
    flex-shrink: 0;
  }
`;

const GitHubIcon = () => (
  <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M12 0C5.374 0 0 5.373 0 12C0 17.302 3.438 21.8 8.207 23.387C8.806 23.498 9 23.126 9 22.81V20.576C5.662 21.302 4.967 19.16 4.967 19.16C4.421 17.773 3.634 17.404 3.634 17.404C2.545 16.659 3.717 16.675 3.717 16.675C4.922 16.759 5.556 17.912 5.556 17.912C6.626 19.746 8.363 19.216 9.048 18.909C9.155 18.134 9.466 17.604 9.81 17.305C7.145 17 4.343 15.971 4.343 11.374C4.343 10.063 4.812 8.993 5.579 8.153C5.455 7.85 5.044 6.629 5.696 4.977C5.696 4.977 6.704 4.655 8.997 6.207C9.954 5.941 10.98 5.808 12 5.803C13.02 5.808 14.047 5.941 15.006 6.207C17.297 4.655 18.303 4.977 18.303 4.977C18.956 6.63 18.545 7.851 18.421 8.153C19.19 8.993 19.656 10.064 19.656 11.374C19.656 15.983 16.849 16.998 14.177 17.295C14.607 17.667 15 18.397 15 19.517V22.81C15 23.129 15.192 23.504 15.801 23.386C20.566 21.797 24 17.3 24 12C24 5.373 18.627 0 12 0Z" fill="white"/>
  </svg>
);

const MenuIcon = () => (
  <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M3 12H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <path d="M3 6H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <path d="M3 18H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
  </svg>
);

const CloseIcon = () => (
  <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M18 6L6 18" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    <path d="M6 6L18 18" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
  </svg>
);

export function Navigation() {
  const pathname = usePathname();
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

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

  useEffect(() => {
    setIsMobileMenuOpen(false);
  }, [pathname]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (isMobileMenuOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
    return () => {
      document.body.style.overflow = "";
    };
  }, [isMobileMenuOpen]);

  const closeMobileMenu = () => setIsMobileMenuOpen(false);

  return (
    <>
      <NavContainer aria-label="Main navigation">
        <HamburgerButton
          onClick={() => setIsMobileMenuOpen(true)}
          aria-label="Open menu"
        >
          <MenuIcon />
        </HamburgerButton>

        <DesktopNavItems>
          <NavItemLink href="/" $isActive={pathname === "/"}>
            About
          </NavItemLink>
          <NavItemLink href="/leaderboard" $isActive={pathname === "/leaderboard"}>
            Leaderboard
          </NavItemLink>
          <NavItemLink href="/profile" $isActive={pathname === "/profile" || pathname.startsWith("/u/")}>
            Profile
          </NavItemLink>
          <NavItemBase
            as="a"
            href="https://github.com/junhoyeo/tokscale"
            target="_blank"
            rel="noopener noreferrer"
            $isActive={false}
          >
            GitHub
          </NavItemBase>
        </DesktopNavItems>

        <DesktopAuthSection>
          {isLoading ? (
            <LoadingSkeleton />
          ) : user ? (
            <ActionMenu>
              <ActionMenu.Anchor>
                <ProfileButton aria-label={`User menu for ${user.username}`}>
                  <Avatar
                    src={user.avatarUrl || `https://github.com/${user.username}.png`}
                    alt={user.username}
                    size={128}
                    style={{ width: "100%", height: "100%" }}
                  />
                </ProfileButton>
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
                        window.location.href = "/leaderboard";
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
            <SignInButton href="/api/auth/github" aria-label="Sign in with GitHub">
              <SignInIcon>
                <GitHubIcon />
              </SignInIcon>
              <SignInTextFull>Sign in with GitHub</SignInTextFull>
              <SignInTextCompact>Sign in</SignInTextCompact>
            </SignInButton>
          )}
        </DesktopAuthSection>
      </NavContainer>

      {/* Mobile Menu */}
      <MobileMenuOverlay $isOpen={isMobileMenuOpen} onClick={closeMobileMenu} />
      <MobileMenuPanel $isOpen={isMobileMenuOpen} onClick={(e) => e.stopPropagation()}>
        <MobileMenuHeader>
          <Link href="/" onClick={closeMobileMenu} style={{ textDecoration: 'none' }}>
            <MobileMenuLogoWrapper>
              <MobileMenuLogoImage
                src="/assets/hero-logo.svg"
                alt="Tokscale"
                fill
              />
            </MobileMenuLogoWrapper>
          </Link>
          <MobileCloseButton onClick={closeMobileMenu} aria-label="Close menu">
            <CloseIcon />
          </MobileCloseButton>
        </MobileMenuHeader>

        <MobileNavLinks>
          <MobileNavLinkItem href="/" $isActive={pathname === "/"} onClick={closeMobileMenu}>
            About
          </MobileNavLinkItem>
          <MobileNavLinkItem href="/leaderboard" $isActive={pathname === "/leaderboard"} onClick={closeMobileMenu}>
            Leaderboard
          </MobileNavLinkItem>
          <MobileNavLinkItem href="/profile" $isActive={pathname === "/profile" || pathname.startsWith("/u/")} onClick={closeMobileMenu}>
            Profile
          </MobileNavLinkItem>
          <MobileNavLinkExternal
            href="https://github.com/junhoyeo/tokscale"
            target="_blank"
            rel="noopener noreferrer"
            $isActive={false}
            onClick={closeMobileMenu}
          >
            GitHub ↗
          </MobileNavLinkExternal>
        </MobileNavLinks>

        <MobileAuthSection>
          {isLoading ? null : user ? (
            <>
              <MobileUserCard>
                <Avatar
                  src={user.avatarUrl || `https://github.com/${user.username}.png`}
                  alt={user.username}
                  size={40}
                />
                <MobileUserDetails>
                  <MobileDisplayName>{user.displayName || user.username}</MobileDisplayName>
                  <MobileUsername>@{user.username}</MobileUsername>
                </MobileUserDetails>
              </MobileUserCard>
              <MobileUserActions>
                <MobileUserAction href={`/u/${user.username}`} onClick={closeMobileMenu}>
                  <PersonIcon size={16} />
                  Your Profile
                </MobileUserAction>
                <MobileUserAction href="/settings" onClick={closeMobileMenu}>
                  <GearIcon size={16} />
                  Settings
                </MobileUserAction>
                <MobileSignOutButton
                  onClick={async () => {
                    await fetch("/api/auth/logout", { method: "POST" });
                    setUser(null);
                    closeMobileMenu();
                    window.location.href = "/leaderboard";
                  }}
                >
                  <SignOutIcon size={16} />
                  Sign Out
                </MobileSignOutButton>
              </MobileUserActions>
            </>
          ) : (
            <MobileSignInButton href="/api/auth/github">
              <SignInIcon>
                <GitHubIcon />
              </SignInIcon>
              <MobileSignInText>Sign in with GitHub</MobileSignInText>
            </MobileSignInButton>
          )}
        </MobileAuthSection>
      </MobileMenuPanel>
    </>
  );
}
