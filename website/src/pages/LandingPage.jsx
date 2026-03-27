import { useEffect, useRef, useState } from 'react';
import {
  getDoWhizApiBaseUrl,
  getOrCreateSessionId,
  persistAttributionFromLocation,
  trackAnalyticsEvent
} from '../analytics';
import { supabase } from '../app/supabaseClient';
import oliverImg from '../assets/Oliver.jpg';
import MouseField from '../components/landing/MouseField';
import {
  getNextThemeSwitch,
  getThemeForLocalTime,
  shouldEnableMouseField
} from '../components/landing/mouseFieldUtils';
import { getLandingContent } from './landingContent';

const SITE_URL = 'https://dowhiz.com';
const LOGO_URL = `${SITE_URL}/assets/DoWhiz.svg`;
const SUPPORT_EMAIL = 'admin@dowhiz.com';
const ORG_NAME = 'DoWhiz';
const CN_PATH_PREFIX = '/cn';
const LANDING_DASHBOARD_SUFFIX = '?loggedIn=true#section-overview';
const LANDING_SETTINGS_SUFFIX = '#section-settings';
const AUTHENTICATED_SETTINGS_SUFFIX = '?loggedIn=true#section-settings';
const LANDING_PAGE_VARIANT = 'oliver_channel_first_v1';
const OAUTH_ENDPOINTS = {
  discord: '/auth/discord',
  slack: '/auth/slack',
  github: '/auth/github',
  notion: '/auth/notion',
  lark: '/auth/lark'
};

const isCnPath = (pathname = '/') =>
  pathname === CN_PATH_PREFIX || pathname === `${CN_PATH_PREFIX}/` || pathname.startsWith(`${CN_PATH_PREFIX}/`);

const getLocalizedAuthPath = (
  suffix = '',
  pathname = typeof window !== 'undefined' ? window.location.pathname : '/'
) => `${isCnPath(pathname) ? CN_PATH_PREFIX : ''}/auth/index.html${suffix}`;

const getLocalizedDashboardPath = (
  pathname = typeof window !== 'undefined' ? window.location.pathname : '/'
) => getLocalizedAuthPath(LANDING_DASHBOARD_SUFFIX, pathname);

const updateMetaContent = (selector, content) => {
  if (typeof document === 'undefined' || !content) {
    return;
  }

  const node = document.querySelector(selector);
  if (node) {
    node.setAttribute('content', content);
  }
};

const updateLinkHref = (selector, href) => {
  if (typeof document === 'undefined' || !href) {
    return;
  }

  const node = document.querySelector(selector);
  if (node) {
    node.setAttribute('href', href);
  }
};

function LandingPage({ locale }) {
  const pathname = typeof window !== 'undefined' ? window.location.pathname : '/';
  const pageLocale = locale || (isCnPath(pathname) ? 'zh-CN' : 'en-US');
  const isChinesePage = pageLocale === 'zh-CN';
  const content = getLandingContent(pageLocale);
  const [theme, setTheme] = useState(() => getThemeForLocalTime());
  const [enableMouseField, setEnableMouseField] = useState(false);
  const [user, setUser] = useState(null);
  const [authStatus, setAuthStatus] = useState('checking');
  const [activeToolKey, setActiveToolKey] = useState(null);
  const [showUserMenu, setShowUserMenu] = useState(false);
  const [navHidden, setNavHidden] = useState(false);
  const userMenuRef = useRef(null);
  const lastScrollY = useRef(0);
  const localizedHomePath = content.nav.homePath;
  const isAuthenticated = authStatus === 'authenticated' && Boolean(user);

  useEffect(() => {
    if (typeof window === 'undefined') {
      return;
    }
    const { hash, pathname } = window.location;
    if (!hash || pathname.startsWith('/auth')) {
      return;
    }
    const params = new URLSearchParams(hash.substring(1));
    const hasTokens = params.get('access_token') && params.get('refresh_token');
    const hasError = params.get('error') || params.get('error_description');
    if (hasTokens || hasError) {
      window.location.replace(getLocalizedAuthPath(hash, pathname));
    }
  }, []);

  useEffect(() => {
    if (typeof window === 'undefined') {
      return;
    }

    if (authStatus !== 'anonymous') {
      return;
    }

    persistAttributionFromLocation();
    const sessionId = getOrCreateSessionId();
    trackAnalyticsEvent(
      'landing_page_view',
      {
        landing_page_variant: LANDING_PAGE_VARIANT,
        landing_page_variant_legacy: 'oliver_consumer_v1',
        language: pageLocale
      },
      {
        eventKey: `landing_page_view:${sessionId}:${window.location.pathname}`
      }
    );
  }, [authStatus, pageLocale]);

  useEffect(() => {
    if (typeof document === 'undefined') {
      return;
    }

    document.documentElement.lang = content.metadata.htmlLang;
    document.title = content.metadata.title;

    updateMetaContent('meta[name="description"]', content.metadata.description);
    updateMetaContent('meta[property="og:title"]', content.metadata.title);
    updateMetaContent('meta[property="og:description"]', content.metadata.description);
    updateMetaContent('meta[property="og:url"]', content.metadata.canonicalUrl);
    updateMetaContent('meta[property="og:locale"]', content.metadata.ogLocale);
    updateMetaContent('meta[name="twitter:title"]', content.metadata.title);
    updateMetaContent('meta[name="twitter:description"]', content.metadata.description);
    updateMetaContent('meta[name="theme-color"]', content.metadata.themeColor);
    updateLinkHref('link[rel="canonical"]', content.metadata.canonicalUrl);
  }, [content.metadata]);

  useEffect(() => {
    if (typeof window === 'undefined') {
      return undefined;
    }

    const scrollToHashTarget = () => {
      const hash = window.location.hash;
      if (!hash) {
        return;
      }

      const targetId = decodeURIComponent(hash.replace(/^#/, ''));
      const target = document.getElementById(targetId);
      if (target) {
        target.scrollIntoView({ behavior: 'auto', block: 'start' });
      }
    };

    const timeoutId = window.setTimeout(scrollToHashTarget, 120);
    window.addEventListener('hashchange', scrollToHashTarget);

    return () => {
      window.clearTimeout(timeoutId);
      window.removeEventListener('hashchange', scrollToHashTarget);
    };
  }, []);

  useEffect(() => {
    const handleClickOutside = (event) => {
      if (userMenuRef.current && !userMenuRef.current.contains(event.target)) {
        setShowUserMenu(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  useEffect(() => {
    let isActive = true;

    const syncSession = (session) => {
      if (!isActive) {
        return;
      }

      const currentUser = session?.user ?? null;
      setUser(currentUser);

      if (currentUser) {
        setAuthStatus('authenticated');
        return;
      }

      setAuthStatus('anonymous');
    };

    supabase.auth
      .getSession()
      .then(({ data: { session } }) => {
        syncSession(session);
      })
      .catch((error) => {
        console.error('App: Failed to load Supabase session', error);
        if (isActive) {
          setUser(null);
          setAuthStatus('anonymous');
        }
      });

    const {
      data: { subscription }
    } = supabase.auth.onAuthStateChange((_event, session) => {
      syncSession(session);
    });

    return () => {
      isActive = false;
      subscription.unsubscribe();
    };
  }, []);

  useEffect(() => {
    let timeoutId;

    const updateTheme = () => {
      setTheme(getThemeForLocalTime());
    };

    const scheduleNextSwitch = () => {
      const now = new Date();
      const nextSwitch = getNextThemeSwitch(now);
      const delay = Math.max(nextSwitch.getTime() - now.getTime(), 0);

      timeoutId = window.setTimeout(() => {
        updateTheme();
        scheduleNextSwitch();
      }, delay);
    };

    updateTheme();
    scheduleNextSwitch();

    return () => {
      if (timeoutId) {
        window.clearTimeout(timeoutId);
      }
    };
  }, []);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  useEffect(() => {
    const handleScroll = () => {
      const currentScrollY = window.scrollY;
      const scrollThreshold = 100;

      if (currentScrollY > lastScrollY.current && currentScrollY > scrollThreshold) {
        setNavHidden(true);
      } else {
        setNavHidden(false);
      }
      lastScrollY.current = currentScrollY;
    };

    window.addEventListener('scroll', handleScroll, { passive: true });
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  useEffect(() => {
    if (!shouldEnableMouseField()) {
      return undefined;
    }

    let idleId;
    let timeoutId;

    const revealMouseField = () => {
      setEnableMouseField(true);
    };

    if ('requestIdleCallback' in window) {
      idleId = window.requestIdleCallback(revealMouseField, { timeout: 1500 });
      return () => {
        if (typeof window.cancelIdleCallback === 'function') {
          window.cancelIdleCallback(idleId);
        }
      };
    }

    timeoutId = window.setTimeout(revealMouseField, 800);
    return () => {
      if (timeoutId) {
        window.clearTimeout(timeoutId);
      }
    };
  }, []);

  const buildMailtoLink = (email, subject, body) => {
    const encodedSubject = encodeURIComponent(subject);
    const encodedBody = encodeURIComponent(body);
    return `mailto:${email}?subject=${encodedSubject}&body=${encodedBody}`;
  };

  const structuredData = {
    '@context': 'https://schema.org',
    '@graph': [
      {
        '@type': 'Organization',
        '@id': `${SITE_URL}/#organization`,
        name: ORG_NAME,
        url: `${SITE_URL}/`,
        logo: LOGO_URL,
        email: `mailto:${SUPPORT_EMAIL}`,
        contactPoint: [
          {
            '@type': 'ContactPoint',
            email: SUPPORT_EMAIL,
            contactType: 'customer support',
            availableLanguage: isChinesePage ? ['Chinese', 'English'] : ['English']
          }
        ],
        sameAs: ['https://github.com/KnoWhiz/DoWhiz']
      },
      {
        '@type': 'FAQPage',
        '@id': `${content.metadata.canonicalUrl}#faq`,
        url: content.metadata.canonicalUrl,
        inLanguage: content.metadata.htmlLang,
        mainEntity: content.faqItems.map((item) => ({
          '@type': 'Question',
          name: item.question,
          acceptedAnswer: {
            '@type': 'Answer',
            text: item.answer
          }
        }))
      }
    ]
  };

  const oliverContactHref = buildMailtoLink('oliver@dowhiz.com', content.hero.contactSubject, content.hero.contactBody);
  const settingsHref = isAuthenticated
    ? getLocalizedAuthPath(AUTHENTICATED_SETTINGS_SUFFIX, pathname)
    : getLocalizedAuthPath(LANDING_SETTINGS_SUFFIX, pathname);
  const manageSetupLabel = isAuthenticated
    ? content.hero.manageAuthenticated
    : content.hero.manageAnonymous;
  const toolHint = isAuthenticated ? content.hero.toolsHintAuthenticated : content.hero.toolsHintAnonymous;

  const trackCtaClick = (eventName, properties) => {
    trackAnalyticsEvent(eventName, properties);
  };

  const startProviderConnect = async (provider) => {
    const endpoint = OAUTH_ENDPOINTS[provider];
    if (!endpoint) {
      window.location.href = settingsHref;
      return;
    }

    try {
      const {
        data: { session }
      } = await supabase.auth.getSession();

      if (!session?.access_token) {
        window.location.href = settingsHref;
        return;
      }

      const response = await fetch(`${getDoWhizApiBaseUrl()}${endpoint}`, {
        headers: { Authorization: `Bearer ${session.access_token}` }
      });
      const data = await response.json().catch(() => null);

      if (!response.ok || !data?.redirect_url) {
        throw new Error(data?.error || `Failed to start ${provider} connection`);
      }

      window.location.href = data.redirect_url;
    } catch (error) {
      console.error(`Landing: failed to start ${provider} connect flow`, error);
      setActiveToolKey(null);
      window.location.href = settingsHref;
    }
  };

  const getHeroToolActionLabel = (toolKey) => {
    if (activeToolKey === toolKey) {
      return content.hero.actionLabels.loading;
    }

    return isAuthenticated ? content.hero.actionLabels.connect : content.hero.actionLabels.setup;
  };

  const handleHeroToolAction = async (tool) => {
    if (activeToolKey) {
      return;
    }

    const actionType =
      tool.key === 'email'
        ? 'mailto'
        : isAuthenticated
          ? 'oauth'
          : 'setup';

    trackCtaClick('hero_tool_action_click', {
      cta_location: 'hero_tool_grid',
      cta_text: tool.label,
      tool: tool.key,
      action_type: actionType,
      landing_page_variant: LANDING_PAGE_VARIANT
    });

    if (typeof window === 'undefined') {
      return;
    }

    if (!isAuthenticated) {
      window.location.href = settingsHref;
      return;
    }

    setActiveToolKey(tool.key);
    await startProviderConnect(tool.key);
  };

  const [openFaq, setOpenFaq] = useState(null);
  const toggleFaq = (idx) => setOpenFaq((prev) => (prev === idx ? null : idx));

  return (
    <div className={`app-container landing-page${isChinesePage ? ' landing-page-cn' : ''}`}>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(structuredData) }}
      />
      <div className="content-layer">
        <nav className={`navbar${navHidden ? ' nav-hidden' : ''}`}>
          <div className="nav-content">
            <a href={localizedHomePath} className="logo">
              <img src="/assets/DoWhiz.svg" alt="" className="brand-mark" aria-hidden="true" />
              <span>Do<span className="text-gradient">Whiz</span></span>
            </a>
            <div className="nav-links">
              {content.nav.links.map((link) => (
                <a key={link.href} href={link.href} className="nav-btn">
                  {link.label}
                </a>
              ))}
            </div>
            <div className="nav-actions">
              <div className="social-links">
                <a
                  href="https://github.com/KnoWhiz/DoWhiz"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="btn-small"
                  aria-label={content.nav.githubAriaLabel}
                >
                  <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
                  </svg>
                </a>
                <a
                  href="https://discord.gg/7ucnweCKk8"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="btn-small"
                  aria-label={content.nav.discordAriaLabel}
                >
                  <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"></path>
                  </svg>
                </a>
                <a
                  className="btn-small"
                  href={oliverContactHref}
                  aria-label={content.nav.contactAriaLabel}
                  onClick={() =>
                    trackCtaClick('secondary_cta_click', {
                      cta_location: 'nav_contact',
                      cta_text: content.nav.contactAriaLabel
                    })
                  }
                >
                  <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
                    <polyline points="22,6 12,13 2,6"></polyline>
                  </svg>
                </a>
                {user ? (
                  <div className="user-menu-container" ref={userMenuRef}>
                    <button
                      type="button"
                      className="user-profile-btn"
                      onClick={() => setShowUserMenu(!showUserMenu)}
                      aria-label={user.user_metadata?.full_name || user.email || (isChinesePage ? '用户菜单' : 'User menu')}
                      title={user.user_metadata?.full_name || user.email || (isChinesePage ? '用户菜单' : 'User menu')}
                    >
                      <img
                        src={user.user_metadata?.avatar_url || user.user_metadata?.picture}
                        alt={user.user_metadata?.full_name || user.email}
                        className="user-avatar"
                      />
                    </button>
                    {showUserMenu && (
                      <div className="user-dropdown">
                        <a
                          href={getLocalizedDashboardPath(pathname)}
                          className="dropdown-item"
                          onClick={async (e) => {
                            e.preventDefault();
                            const {
                              data: { session }
                            } = await supabase.auth.getSession();
                            window.location.href = session
                              ? getLocalizedDashboardPath(window.location.pathname)
                              : getLocalizedAuthPath('', window.location.pathname);
                          }}
                        >
                          <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="2" fill="none">
                            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                          </svg>
                          {content.nav.dashboard}
                        </a>
                        <button
                          className="dropdown-item"
                          onClick={async () => {
                            await supabase.auth.signOut();
                            setUser(null);
                            setShowUserMenu(false);
                          }}
                        >
                          <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="2" fill="none">
                            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
                            <polyline points="16 17 21 12 16 7" />
                            <line x1="21" y1="12" x2="9" y2="12" />
                          </svg>
                          {content.nav.signOut}
                        </button>
                      </div>
                    )}
                  </div>
                ) : (
                  <a
                    className="btn-small"
                    href={getLocalizedAuthPath('', pathname)}
                    aria-label={content.nav.signIn}
                    onClick={() =>
                      trackCtaClick('primary_cta_click', {
                        cta_location: 'nav_sign_in',
                        cta_text: content.nav.signIn
                      })
                    }
                  >
                    <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
                      <circle cx="12" cy="7" r="4"></circle>
                    </svg>
                  </a>
                )}
              </div>
            </div>
          </div>
        </nav>

        <section id="channels" className="hero-section">
          {enableMouseField ? <MouseField theme={theme} /> : null}
          <div className="halo-effect"></div>
          <div className="container hero-content hero-shell">
            <div className="hero-copy hero-copy-compact">
              <p className="hero-eyebrow">{content.hero.eyebrow}</p>
              <h1 className="hero-title">{content.hero.title}</h1>
              <p className="hero-subtitle">{content.hero.subtitle}</p>
              <div className="hero-cta-row">
                <a
                  className="btn btn-primary hero-primary-cta"
                  href={oliverContactHref}
                  onClick={() =>
                    trackCtaClick('primary_cta_click', {
                      cta_location: 'hero_primary_email',
                      cta_text: content.hero.primaryCta,
                      landing_page_variant: LANDING_PAGE_VARIANT
                    })
                  }
                >
                  {content.hero.primaryCta}
                </a>
                <a
                  className="btn btn-secondary hero-secondary-cta"
                  href="#watch"
                  onClick={() =>
                    trackCtaClick('secondary_cta_click', {
                      cta_location: 'hero_secondary_watch',
                      cta_text: content.hero.secondaryCta,
                      landing_page_variant: LANDING_PAGE_VARIANT
                    })
                  }
                >
                  {content.hero.secondaryCta}
                </a>
              </div>
              <p className="hero-caption">{content.hero.caption}</p>
            </div>

            <div className="hero-product-card">
              <div className="hero-product-head">
                <div className="hero-product-heading">
                  <span className="hero-panel-kicker">{content.hero.toolsEyebrow}</span>
                  <p>{toolHint}</p>
                </div>
                <div className="hero-operator-chip">
                  <div className="hero-operator-portrait">
                    <img src={oliverImg} alt="Oliver" className="hero-portrait" />
                  </div>
                  <div className="hero-operator-copy">
                    <strong>Oliver</strong>
                    <span>{isChinesePage ? '可信 AI operator' : 'Trusted AI operator'}</span>
                  </div>
                </div>
              </div>

              <article className="hero-direct-card">
                <div className="hero-direct-copy">
                  <span className="hero-panel-kicker">{content.hero.directEyebrow}</span>
                  <h2>{content.hero.directTitle}</h2>
                  <p>{content.hero.directDescription}</p>
                  <div className="hero-direct-meta">
                    <span className="hero-direct-badge">{content.hero.directBadge}</span>
                    <span className="hero-direct-subnote">{content.hero.directSubnote}</span>
                  </div>
                </div>
                <a
                  className="btn btn-primary hero-direct-cta"
                  href={oliverContactHref}
                  onClick={() =>
                    trackCtaClick('primary_cta_click', {
                      cta_location: 'hero_direct_email',
                      cta_text: content.hero.directActionLabel,
                      landing_page_variant: LANDING_PAGE_VARIANT
                    })
                  }
                >
                  {content.hero.directActionLabel}
                </a>
              </article>

              <div className="hero-tool-grid" role="group" aria-label={content.hero.toolsEyebrow}>
                {content.hero.tools.map((tool) => (
                  <button
                    key={tool.key}
                    type="button"
                    className="hero-tool-card"
                    style={{ '--tool-accent': tool.accent }}
                    onClick={() => handleHeroToolAction(tool)}
                    disabled={Boolean(activeToolKey)}
                  >
                    <div className="hero-tool-card-top">
                      <span className="hero-tool-badge" aria-hidden="true">
                        {tool.monogram}
                      </span>
                      <span className="hero-tool-action">{getHeroToolActionLabel(tool.key)}</span>
                    </div>
                    <div className="hero-tool-card-copy">
                      <strong>{tool.label}</strong>
                      <span className="hero-tool-status">{tool.availability}</span>
                      <span>{tool.description}</span>
                    </div>
                  </button>
                ))}
              </div>

              <div className="hero-channel-footnote">
                <p>{content.hero.toolsFootnote}</p>
                <a
                  href={settingsHref}
                  onClick={() =>
                    trackCtaClick('secondary_cta_click', {
                      cta_location: 'hero_manage_setup',
                      cta_text: manageSetupLabel,
                      landing_page_variant: LANDING_PAGE_VARIANT
                    })
                  }
                >
                  {manageSetupLabel}
                </a>
              </div>
            </div>
          </div>
        </section>

        <section id="watch" className="section demo-showcase-section">
          <div className="container">
            <div className="section-heading-shell">
              <span className="section-kicker">{content.demo.eyebrow}</span>
              <h2 className="section-title section-title-left">{content.demo.title}</h2>
              <p className="section-intro section-intro-left">{content.demo.intro}</p>
            </div>

            <div className="demo-showcase-grid">
              <article className="demo-feature-card">
                <div className="demo-card-head">
                  <div>
                    <h3>{content.demo.desktopTitle}</h3>
                    <p>{content.demo.desktopDescription}</p>
                  </div>
                  <a
                    className="demo-inline-link"
                    href={content.demo.desktopVideoHref}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {content.demo.desktopCta}
                  </a>
                </div>
                <div className="frame-shell frame-landscape">
                  <iframe
                    src={`https://www.youtube.com/embed/${content.demo.desktopVideoId}?rel=0`}
                    title={content.demo.desktopTitle}
                    loading="lazy"
                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                    referrerPolicy="strict-origin-when-cross-origin"
                    allowFullScreen
                  ></iframe>
                </div>
              </article>

              <aside className="demo-short-rail">
                <div className="demo-short-head">
                  <h3>{content.demo.shortsTitle}</h3>
                  <p>{content.demo.shortsDescription}</p>
                </div>
                <div className="demo-short-grid">
                  {content.demo.shorts.map((item) => (
                    <article key={item.videoId} className="demo-short-card">
                      <div className="frame-shell frame-portrait">
                        <iframe
                          src={`https://www.youtube.com/embed/${item.videoId}?rel=0`}
                          title={item.title}
                          loading="lazy"
                          allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                          referrerPolicy="strict-origin-when-cross-origin"
                          allowFullScreen
                        ></iframe>
                      </div>
                      <a
                        className="demo-short-link"
                        href={item.href}
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        {item.title}
                      </a>
                    </article>
                  ))}
                </div>
              </aside>
            </div>
          </div>
        </section>

        <section id="examples" className="section example-showcase-section">
          <div className="container story-stack">
            <div className="section-heading-shell">
              <span className="section-kicker">{content.examples.eyebrow}</span>
              <h2 className="section-title section-title-left">{content.examples.title}</h2>
              <p className="section-intro section-intro-left">{content.examples.intro}</p>
            </div>

            <div className="example-card-grid">
              {content.examples.cards.map((item) => (
                <article key={item.title} className="example-card">
                  <span className="example-card-tag">{item.tag}</span>
                  <h3>{item.title}</h3>
                  <p>{item.description}</p>
                </article>
              ))}
            </div>

            <aside className="control-band">
              <div className="control-band-copy">
                <span className="section-kicker">{content.control.eyebrow}</span>
                <h3>{content.control.title}</h3>
              </div>
              <ul className="control-band-list">
                {content.control.points.map((point) => (
                  <li key={point}>{point}</li>
                ))}
              </ul>
            </aside>
          </div>
        </section>

        <section id="faq" className="section faq-section">
          <div className="container">
            <div className="section-heading-shell">
              <span className="section-kicker">{content.labels.faqEyebrow}</span>
              <h2 className="section-title section-title-left">{content.labels.faqTitle}</h2>
              <p className="section-intro section-intro-left">{content.labels.faqIntro}</p>
            </div>
            <div className="faq-accordion faq-compact">
              {content.faqItems.map((item, idx) => {
                const isOpen = openFaq === idx;
                return (
                  <article key={item.question} className={`faq-accordion-item ${isOpen ? 'open' : ''}`}>
                    <button
                      type="button"
                      className="faq-accordion-header"
                      onClick={() => toggleFaq(idx)}
                      aria-expanded={isOpen}
                      aria-controls={`faq-panel-${idx}`}
                    >
                      <span className="faq-question">{item.question}</span>
                      <span className="faq-toggle" aria-hidden="true">
                        {isOpen ? '−' : '+'}
                      </span>
                    </button>
                    <div
                      id={`faq-panel-${idx}`}
                      className="faq-accordion-panel"
                      style={{ display: isOpen ? 'block' : 'none' }}
                    >
                      <p>{item.answer}</p>
                    </div>
                  </article>
                );
              })}
            </div>
            <div className="faq-link-row">
              <a className="faq-text-link" href="https://www.dowhiz.com/help-center/">
                {content.labels.faqLinkLabel}
              </a>
            </div>
          </div>
        </section>

        <footer className="site-footer">
          <div className="container footer-content">
            <div className="footer-brand">
              <a href={localizedHomePath} className="footer-logo">
                <img src="/assets/DoWhiz.svg" alt="" className="footer-brand-mark" aria-hidden="true" />
                <span>Do<span className="text-gradient">Whiz</span></span>
              </a>
              <p className="footer-tagline">{content.labels.footerTagline}</p>
              <div className="footer-pill">{content.labels.footerPill}</div>
            </div>
            <div className="footer-links">
              <span className="footer-title">{content.labels.footerTitle}</span>
              <div className="footer-link-grid">
                {content.footerLinks.map((link) => (
                  <a key={link.href} href={link.href} className="footer-link">
                    {link.label}
                  </a>
                ))}
                <a href={`mailto:${SUPPORT_EMAIL}`} className="footer-link">
                  {isChinesePage ? '联系 Oliver' : 'Contact Oliver'}
                </a>
              </div>
            </div>
          </div>
          <div className="container footer-bottom">
            <span>
              &copy; {new Date().getFullYear()} DoWhiz. {isChinesePage ? '保留所有权利。' : 'All rights reserved.'}
            </span>
            <span>{content.labels.footerBottomSecondary}</span>
          </div>
        </footer>
      </div>
    </div>
  );
}

export default LandingPage;
