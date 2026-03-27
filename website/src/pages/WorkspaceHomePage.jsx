import { useEffect } from 'react';
import { Link } from 'react-router-dom';

const DASHBOARD_WORKSPACE_ANCHOR = '/auth/index.html?loggedIn=true#section-workspace';

function WorkspaceHomePage() {
  useEffect(() => {
    if (typeof window === 'undefined') {
      return undefined;
    }

    const timeoutId = window.setTimeout(() => {
      window.location.replace(DASHBOARD_WORKSPACE_ANCHOR);
    }, 250);

    return () => window.clearTimeout(timeoutId);
  }, []);

  return (
    <main className="route-shell route-shell-workspace">
      <div className="route-card route-card-workspace-compact">
        <p className="route-kicker">Legacy route</p>
        <h1>Redirecting to your Oliver setup</h1>
        <p>
          This older workspace preview now lives inside your personal dashboard. You will be redirected automatically.
        </p>
        <p className="workspace-inline-note">
          The main onboarding flow now starts with connected apps, first tasks, and memory inside the Oliver dashboard.
        </p>

        <div className="route-actions">
          <a className="btn btn-primary" href={DASHBOARD_WORKSPACE_ANCHOR}>
            Open Oliver setup
          </a>
          <Link className="btn btn-secondary" to="/">
            Back to landing
          </Link>
        </div>
      </div>
    </main>
  );
}

export default WorkspaceHomePage;
