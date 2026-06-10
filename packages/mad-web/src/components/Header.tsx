export function Header() {
  return (
    <header className="header">
      <div className="header-brand">
        <img src="/logo.png" alt="Operation M.A.D. logo" className="header-logo" />
        <div>
          <h1 className="header-title">Operation M.A.D.</h1>
          <p className="header-subtitle">Mobile MDM Vendor Evaluation — iOS & Android</p>
        </div>
      </div>
      <style>{`
        .header {
          background: linear-gradient(135deg, var(--mad-navy) 0%, var(--mad-blue) 100%);
          color: white;
          padding: 1.5rem 2rem;
          border-bottom: 3px solid var(--mad-cyan);
          box-shadow: 0 4px 20px rgba(10, 22, 40, 0.3);
        }
        .header-brand {
          display: flex;
          align-items: center;
          gap: 1.25rem;
          max-width: 1200px;
          margin: 0 auto;
        }
        .header-logo {
          width: 72px;
          height: 72px;
          border-radius: 8px;
          object-fit: cover;
          border: 2px solid var(--mad-cyan);
          box-shadow: 0 0 16px rgba(0, 180, 216, 0.4);
        }
        .header-title {
          margin: 0;
          font-size: 1.75rem;
          font-weight: 700;
          letter-spacing: 0.02em;
        }
        .header-subtitle {
          margin: 0.25rem 0 0;
          font-size: 0.95rem;
          color: var(--mad-silver);
          opacity: 0.9;
        }
      `}</style>
    </header>
  );
}
