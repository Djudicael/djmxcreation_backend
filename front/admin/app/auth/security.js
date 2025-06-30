import Keycloak from "https://cdn.jsdelivr.net/npm/keycloak-js@26.2.0/+esm";
const keycloak = new Keycloak({
  url: "http://localhost:8080",
  realm: "portfolio",
  clientId: "portfolio",
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
