import { TemplateRenderer, html } from '../utils/template-renderer.js';

export class ServiceComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
    }

    get template() {
        console.log('service');


        return html`
        <section>
            <div class="title">
                <h1> Service that can hel you</h1>
                <p>Lorem ipsum dolor sit amet, consectetur adipisicing elit. Voluptates blanditiis pariatur nostrum tenetur
                    assumenda, doloribus laboriosam beatae quos incidunt atque, quia magnam cumque? Sint iste consequuntur
                    laborum omnis doloribus repudiandae aspernatur animi. Et mollitia dolores perspiciatis dolorum labore vel ex
                    eaque fugiat nam ut itaque consequuntur error quisquam, qui omnis!</p>
            </div>
            <div class="services">
                <div class="service">
                    <div class="icon">
                        <img src="/ressource/icon/DISPLAY.svg" />
                    </div>
                    <h2>Design</h2>
                    Lorem ipsum dolor sit amet, consectetur adipisicing elit. Voluptates blanditiis pariatur nostrum tenetur
                    assumenda, doloribus labo
                </div>
                <div class="service">
                    <div class="icon">
                        <img src="/ressource/icon/DISPLAY.svg" />
                    </div>
                    <h2>Developpement</h2>
                    Lorem ipsum dolor sit amet, consectetur adipisicing elit. Voluptates blanditiis pariatur nostrum tenetur
                    assumenda, doloribus labo
                </div>
                <div class="service">
                    <div class="icon">
                        <img src="/ressource/icon/DISPLAY.svg" />
                    </div>
                    <h2>SEO</h2>
                    Lorem ipsum dolor sit amet, consectetur adipisicing elit. Voluptates blanditiis pariatur nostrum tenetur
                    assumenda, doloribus labo
                </div>
                <div class="service">
                    <div class="icon">
                        <img src="/ressource/icon/DISPLAY.svg" />
                    </div>
                    <h2>Marketing</h2>
                    Lorem ipsum dolor sit amet, consectetur adipisicing elit. Voluptates blanditiis pariatur nostrum tenetur
                    assumenda, doloribus labo
                </div>
                <div class="service">
                    <div class="icon">
                        <img src="/ressource/icon/DISPLAY.svg" />
                    </div>
                    <h2>App develipment</h2>
                    Lorem ipsum dolor sit amet, consectetur adipisicing elit. Voluptates blanditiis pariatur nostrum tenetur
                    assumenda, doloribus labo
                </div>
                <div class="service">
                    <div class="icon">
                        <img src="/ressource/icon/DISPLAY.svg" />
                    </div>
                    <h2>Error Fixig</h2>
                    Lorem ipsum dolor sit amet, consectetur adipisicing elit. Voluptates blanditiis pariatur nostrum tenetur
                    assumenda, doloribus labo
                </div>
            </div>
        </section>
        `;
    }



    connectedCallback() {
        super.connectedCallback();


    }
}