
import app from './app.js';
import http from 'http';


const port = 3009;


const server = http.createServer(app);

server.listen(port,  () =>{
    console.log(`Express server listening on port ${port}`);
});