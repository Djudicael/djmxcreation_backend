import express from 'express';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';


const __dirname = dirname(fileURLToPath(import.meta.url));
console.log(__dirname)

const app = express();
app.disable('x-powered-by');
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

app.use('/style', express.static(join(__dirname, 'style')));
app.use('/app', express.static(join(__dirname, 'app')));
app.use('/lottie', express.static(join(__dirname, 'lottie')));
app.use('/ressource', express.static(join(__dirname, 'ressource')));
app.use('/lib', express.static(join(__dirname, 'lib')));
app.get('/', function (req, res) {
    res.sendFile(join(__dirname + '/index.html'));
});

app.use((err, _, res, _) => {
    res.status(err.status || 500);
    res.send({
        error: {
            status: err.status || 500,
            message: err.message
        }
    })
});

export default app;