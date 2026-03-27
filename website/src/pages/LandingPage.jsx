import { useEffect, useRef, useState } from 'react';
import {
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
const LANDING_DASHBOARD_SUFFIX = '?loggedIn=true#section-workspace';

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
  const localizedHomePath = content.nav.homePath;
  const [theme, setTheme] = useState(() => getThemeForLocalTime());
  const [enableMouseField, setEnableMouseField] = useState(false);
  const [user, setUser] = useState(null);
  const [authStatus, setAuthStatus] = useState('checking');
  const [showUserMenu, setShowUserMenu] = useState(false);
  const [navHidden, setNavHidden] = useState(false);
  const userMenuRef = useRef(null);
  const lastScrollY = useRef(0);
  const authRedirectStartedRef = useRef(false);

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
        landing_page_variant: 'oliver_consumer_v1',
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
        if (!authRedirectStartedRef.current && typeof window !== 'undefined') {
          authRedirectStartedRef.current = true;
          window.location.replace(getLocalizedDashboardPath(window.location.pathname));
        }
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
  const primaryCtaHref = user ? getLocalizedDashboardPath(pathname) : getLocalizedAuthPath('', pathname);
  const secondaryCtaHref = content.hero.secondaryHref || '#use-cases';

  const trackCtaClick = (eventName, properties) => {
    trackAnalyticsEvent(eventName, properties);
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

        <section className="hero-section">
          {enableMouseField ? <MouseField theme={theme} /> : null}
          <div className="halo-effect"></div>
          <div className="container hero-content hero-content-oliver">
            <div className="hero-copy">
              <p className="hero-eyebrow">{content.hero.eyebrow}</p>
              <h1 className="hero-title">{content.hero.title}</h1>
              <p className="hero-subtitle">{content.hero.subtitle}</p>
              <p className="hero-note">{content.hero.note}</p>
              <div className="hero-chip-row" aria-label={isChinesePage ? 'Oliver 工作的主要表面' : 'Primary surfaces Oliver works in'}>
                {content.hero.chips.map((chip) => (
                  <span key={chip} className="hero-chip">
                    {chip}
                  </span>
                ))}
              </div>
              <div className="hero-cta-row">
                <a
                  className="btn btn-primary"
                  href={primaryCtaHref}
                  onClick={() =>
                    trackCtaClick('primary_cta_click', {
                      cta_location: 'hero_primary',
                      cta_text: content.hero.primaryCta
                    })
                  }
                >
                  {content.hero.primaryCta}
                </a>
                <a
                  className="btn btn-secondary"
                  href={secondaryCtaHref}
                  onClick={() =>
                    trackCtaClick('secondary_cta_click', {
                      cta_location: 'hero_secondary',
                      cta_text: content.hero.secondaryCta
                    })
                  }
                >
                  {content.hero.secondaryCta}
                </a>
              </div>
            </div>

            <div className="hero-panel-grid">
              <article className="hero-spotlight-card">
                <div className="hero-spotlight-head">
                  <div className="hero-portrait-wrap">
                    <img src={oliverImg} alt="Oliver" className="hero-portrait" />
                  </div>
                  <div className="hero-spotlight-copy">
                    <span className="hero-panel-kicker">{content.hero.profileEyebrow}</span>
                    <h2>{content.hero.profileTitle}</h2>
                    <p>{content.hero.profileDescription}</p>
                  </div>
                </div>
                <ul className="hero-spotlight-list">
                  {content.hero.profilePoints.map((point) => (
                    <li key={point}>{point}</li>
                  ))}
                </ul>
                <div className="hero-task-sampler">
                  <span className="hero-panel-kicker">{content.hero.exampleTitle}</span>
                  <ul className="hero-task-list">
                    {content.hero.exampleTasks.map((task) => (
                      <li key={task}>{task}</li>
                    ))}
                  </ul>
                </div>
              </article>

              <aside className="hero-onboarding-card" aria-label={content.hero.onboardingAriaLabel}>
                <div className="hero-card-header">
                  <span className="hero-panel-kicker">{content.hero.onboardingTitle}</span>
                  <p>{content.hero.onboardingDescription}</p>
                </div>
                <div className="hero-onboarding-list">
                  {content.onboardingSteps.map((step) => (
                    <div key={step.id} className="hero-onboarding-item">
                      <span className="hero-step-index">{step.id}</span>
                      <div>
                        <h3>{step.title}</h3>
                        <p>{step.desc}</p>
                      </div>
                    </div>
                  ))}
                </div>
              </aside>
            </div>
          </div>
        </section>

        <section id="getting-started" className="section">
          <div className="container">
            <h2 className="section-title">{content.sections.gettingStartedTitle}</h2>
            <p className="section-intro">{content.sections.gettingStartedIntro}</p>
            <div className="step-grid">
              {content.onboardingSteps.map((step) => (
                <article key={step.id} className="step-card">
                  <span className="step-card-index">{step.id}</span>
                  <h3>{step.title}</h3>
                  <p className="step-card-desc">{step.desc}</p>
                  <p className="step-card-detail">{step.detail}</p>
                </article>
              ))}
            </div>
          </div>
        </section>

        <section id="use-cases" className="section">
          <div className="container">
            <h2 className="section-title">{content.sections.useCasesTitle}</h2>
            <p className="section-intro">{content.sections.useCasesIntro}</p>
            <div className="use-case-grid">
              {content.useCases.map((item) => (
                <article key={item.title} className="use-case-card">
                  <span className="use-case-tag">{item.tag}</span>
                  <h3>{item.title}</h3>
                  <p>{item.desc}</p>
                </article>
              ))}
            </div>
          </div>
        </section>

        <section id="surfaces" className="section features-section">
          <div className="container">
            <h2 className="section-title">{content.sections.surfacesTitle}</h2>
            <p className="section-intro">{content.sections.surfacesIntro}</p>
            <div className="surface-grid">
              {content.surfaces.map((surface) => (
                <article key={surface.title} className="surface-card">
                  <div className="surface-card-head">
                    <div className="feature-iconwrap">
                      <img src={surface.icon} alt={surface.title} className="feature-icon" />
                    </div>
                    <span className="use-case-tag">{surface.tag}</span>
                  </div>
                  <h3>{surface.title}</h3>
                  <p>{surface.desc}</p>
                </article>
              ))}
            </div>
          </div>
        </section>

        <section id="control" className="section">
          <div className="container">
            <h2 className="section-title">{content.sections.controlTitle}</h2>
            <p className="section-intro">{content.sections.controlIntro}</p>
            <div className="control-layout">
              <div className="control-card-stack">
                {content.safetyItems.map((item) => (
                  <article key={item.title} className="control-card">
                    <div className="control-card-head">
                      <div className="feature-iconwrap">
                        <img src={item.icon} alt={item.tag} className="feature-icon" />
                      </div>
                      <span className="use-case-tag">{item.tag}</span>
                    </div>
                    <h3>{item.title}</h3>
                    <p>{item.desc}</p>
                    <ul className="control-point-list">
                      {item.points.map((point) => (
                        <li key={point}>{point}</li>
                      ))}
                    </ul>
                  </article>
                ))}
              </div>
              <aside className="control-steps-card">
                <h3>{content.labels.accessPlaybookTitle}</h3>
                <p>{content.labels.accessPlaybookDescription}</p>
                <div className="control-step-list">
                  {content.accessFlowSteps.map((step, index) => (
                    <div key={step.title} className="control-step-item">
                      <span className="control-step-index">{index + 1}</span>
                      <div>
                        <h4>{step.title}</h4>
                        <p>{step.desc}</p>
                      </div>
                    </div>
                  ))}
                </div>
                <a href="/trust-safety/" className="access-playbook-link">
                  {content.labels.accessPlaybookLink}
                </a>
              </aside>
            </div>
          </div>
        </section>

        <section id="faq" className="section faq-section">
          <div className="container">
            <h2 className="section-title">{content.sections.faqTitle}</h2>
            <p className="section-intro">{content.sections.faqIntro}</p>
            <div className="faq-accordion">
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
            <div className="faq-cta">
              <a className="btn btn-secondary" href="https://www.dowhiz.com/help-center/">
                {content.labels.faqCta}
              </a>
            </div>
          </div>
        </section>

        <section id="blog" className="section blog-section">
          <div className="container">
            <div className="blog-header">
              <div>
                <span className="blog-eyebrow">{content.labels.blogEyebrow}</span>
                <h2 className="blog-title">{content.labels.blogTitle}</h2>
                <p className="blog-intro">{content.labels.blogIntro}</p>
              </div>
              <a className="btn btn-secondary blog-header-btn" href="/blog/">
                {content.labels.blogHeaderButton}
              </a>
            </div>
            <div className="blog-grid">
              {content.blogPosts.map((post) => (
                <article key={post.title} className="blog-card" role="article">
                  <div className="blog-meta">
                    <span className="blog-tag">{post.tag}</span>
                    <span className="blog-date">{post.date}</span>
                  </div>
                  <h3>{post.title}</h3>
                  <p>{post.excerpt}</p>
                  <a className="blog-link" href={post.link}>
                    {content.labels.blogLinkLabel}
                    <span aria-hidden="true" className="blog-link-icon"></span>
                  </a>
                </article>
              ))}
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
