import { TemplateRenderer, html } from '../utils/template-renderer.js';

export class WorkComponent extends TemplateRenderer {
    constructor(router) {
        super();
        this.noShadow = true;
    }

    get template() {


        return html`
        <section>
            <div class="title">
                <h1> Some of our finest work.</h1>
                <p>Lorem ipsum dolor sit amet, consectetur adipisicing elit. Voluptates blanditiis pariatur nostrum tenetur
                    assumenda, doloribus laboriosam beatae quos incidunt atque, quia magnam cumque? Sint iste consequuntur
                    laborum omnis doloribus repudiandae aspernatur animi. Et mollitia dolores perspiciatis dolorum labore vel ex
                    eaque fugiat nam ut itaque consequuntur error quisquam, qui omnis!</p>
            </div>
            <div class="portfolio">
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
                <div class="item">
                    <img src="/ressource/123466978_1248923565491630_6047768907212472612_n.jpg" alt="test">
                    <div class="action">
                        <a href="#">Launch</a>
                    </div>
                </div>
            </div>
        </section>
        `;
    }



    connectedCallback() {
        super.connectedCallback();

    }
}