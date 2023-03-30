import express from 'express';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const app = express();

app.disable('x-powered-by');
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

app.use('/', express.static(join(__dirname, 'dist')));
app.use('/ressource', express.static(join(__dirname, 'dist/ressource')));

app.get('/*', function (req, res) {
    res.sendFile(join(__dirname, 'dist/index.html'));
});
app.get('/projects/*', function (req, res) {
    res.sendFile(join(__dirname, 'dist/index.html'));
});


app.get('/js/index.js', (req, res) => {
    res.type('application/javascrip');
    res.sendFile(join(__dirname, 'dist/js/index.js'));
});
export default app;
