// Enable dark mode
user_pref("ui.systemUsesDarkTheme", 1);

// Disable first-run welcome page
user_pref("browser.startup.homepage_override.mstone", "ignore");
user_pref("startup.homepage_welcome_url", "");
user_pref("startup.homepage_welcome_url.additional", "");

// Disable what's new and update pages
user_pref("browser.startup.homepage_override.mstone", "ignore");
user_pref("browser.messaging-system.whatsNewPanel.enabled", false);

// Disable default browser check
user_pref("browser.shell.checkDefaultBrowser", false);

// Disable about:config warning
user_pref("browser.aboutConfig.showWarning", false);

// Disable privacy notice
user_pref("datareporting.policy.dataSubmissionPolicyBypassNotification", true);

// Disable telemetry
user_pref("toolkit.telemetry.reportingpolicy.firstRun", false);

// Disable sidebar
user_pref("sidebar.revamp", false);
user_pref("sidebar.verticalTabs", false);
user_pref("sidebar.new-sidebar.has-used", false);
user_pref("browser.newtabpage.activity-stream.asrouter.userprefs.cfr.features", false);
user_pref("browser.ml.chat.sidebar", false);

// Disable account sync
user_pref("identity.fxaccounts.enabled", false);
user_pref("identity.fxaccounts.tolbar.enabled", false);

// Disable password manager
user_pref("signon.rememberSignons", false);
user_pref("signon.autofillForms", false);

// Disable translate
user_pref("browser.translations.enable", false);
