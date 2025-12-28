/**
 * Footer Component - Application footer
 */

import React from 'react';

export const Footer: React.FC = () => {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="footer">
      <div className="footer-content">
        <div className="footer-left">
          <span className="footer-text">
            &copy; {currentYear} AccuScene Enterprise. All rights reserved.
          </span>
        </div>
        <div className="footer-right">
          <a href="#" className="footer-link">
            Privacy Policy
          </a>
          <span className="footer-separator">|</span>
          <a href="#" className="footer-link">
            Terms of Service
          </a>
          <span className="footer-separator">|</span>
          <a href="#" className="footer-link">
            Support
          </a>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
