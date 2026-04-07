import Keycloak from "keycloak-js";

const KEYCLOAK_URL = import.meta?.env?.KEYCLOAK_URL || "http://localhost:8080";
const KEYCLOAK_REALM = import.meta?.env?.KEYCLOAK_REALM || "portfolio";
const KEYCLOAK_CLIENT_ID = import.meta?.env?.KEYCLOAK_CLIENT_ID || "portfolio";

const keycloak = new Keycloak({
  url: KEYCLOAK_URL,
  realm: KEYCLOAK_REALM,
  clientId: KEYCLOAK_CLIENT_ID,
  "public-client": true,
});

const initKeycloak = (onAuthenticatedCallback) => {
  keycloak
    .init({
      // silentCheckSsoRedirectUri: window.location.origin,
      //   silentCheckSsoRedirectUri: window.location.origin + '/silent-check-sso.html',
      pkceMethod: "S256",
      onLoad: "login-required",
      // onLoad: 'check-sso',
      // checkLoginIframe: false
    })
    .then((authenticated) => {
      if (!authenticated) {
        console.log("user is not authenticated..!");
      }
      onAuthenticatedCallback();
    })
    .catch(console.error);
};

const doLogin = keycloak.login;

const doLogout = keycloak.logout;

const getToken = () => keycloak.token;

const isLoggedIn = () => !!keycloak.token;

const updateToken = (successCallback) =>
  keycloak.updateToken(5).then(successCallback).catch(doLogin);

const getUsername = () => keycloak.tokenParsed?.preferred_username;

const hasRole = (roles) => roles.some((role) => keycloak.hasRealmRole(role));

export { initKeycloak };
