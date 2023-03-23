import { TemplateRenderer, html } from '../utils/template-renderer.js';
import PortfolioApi from '../api/portfolio.api.js';
export class HomeComponent extends TemplateRenderer {
    constructor(router) {
        super();
        this.noShadow = true;
        this.router = router;
        this.instance = new PortfolioApi();
        this.showReel;
        this.projects = [];
    }

    get template() {


        return html`
        <section class="main-content">
            <c-header></c-header>
            <main>
                <div class="cards">
                    <div class="card-single">
                        <div>
                            <h1>54</h1>
                            <span>Users</span>
                        </div>
                        <div> <img src="/ressource/icon/group.svg" /></div>
                    </div>
                    <div class="card-single">
                        <div>
                            <h1>79</h1>
                            <span>PROJECTS</span>
                        </div>
                        <div> <img src="/ressource/icon/folder.svg" /></div>
                    </div>
                </div>
                <div class="recent-grid">
                    <div class="projects">
                        <div class="card">
                            <div class="card-header">
                                <h3>Recent Projects</h3>
                                <button>See all <img class="button-icon" src="/ressource/icon/next.svg" /></button>
                            </div>
                            <div class="card-body">
                                <table>
                                    <thead>
                                        <tr>
                                            <td>Project Title</td>
                                            <td>Style</td>
                                            <td>Client</td>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <tr>
                                            <td>Family Home</td>
                                            <td>Animation</td>
                                            <td>Netflix</td>
                                        </tr>
                                        <tr>
                                            <td>Family Home</td>
                                            <td>Animation</td>
                                            <td>Netflix</td>
                                        </tr>
                                        <tr>
                                            <td>Family Home</td>
                                            <td>Animation</td>
                                            <td>Netflix</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                    <div class="users">
                        <div class="card">
                            <div class="card-header">
                                <h3>New users</h3>
                                <button>See all <img class="button-icon" src="/ressource/icon/next.svg" /></button>
                            </div>
                            <div class="card-body">
                                <div class="user">
                                    <div class="info">
                                        <img src="/ressource/profileUser.jpg" width="40px" height="40px" alt="profile" />
                                        <div>
                                            <h4> Judi test</h4>
                                            <small>CEO/CTO</small>
                                        </div>
                                    </div>
                                </div>
                                <div class="user">
                                    <div class="info">
                                        <img src="/ressource/profileUser.jpg" width="40px" height="40px" alt="profile" />
                                        <div>
                                            <h4> Judi test</h4>
                                            <small>CEO/CTO</small>
                                        </div>
                                    </div>
                                </div>
                                <div class="user">
                                    <div class="info">
                                        <img src="/ressource/profileUser.jpg" width="40px" height="40px" alt="profile" />
                                        <div>
                                            <h4> Judi test</h4>
                                            <small>CEO/CTO</small>
                                        </div>
                                    </div>
                                </div>
        
                            </div>
                        </div>
                    </div>
                </div>
        
            </main>
        
        </section>
        `;
    }

    async getShowReel() {
        const response = await this.instance.getShowReel();
        this.showReel = response.url
        super.render();
    }

    async getProjects() {
        const response = await this.instance.getProjects();
        this.projects.push(...response);
        super.render();
    }

    connectedCallback() {
        super.connectedCallback();
        this.router.updatePageLinks();
        // this.getShowReel();
        // this.getProjects();
    }
}