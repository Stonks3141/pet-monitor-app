import express from 'express';
import session from 'express-session';
import bodyParser from 'body-parser';
import passport from 'passport';
import crypto from 'crypto';
import { authRouter, streamRouter } from './routes';
import fs from 'fs';
import https from 'https';

const credentials = {
  cert: fs.readFileSync('cert.pem'),
  key: fs.readFileSync('key.pem')
};

const app = express();
const port = 8443;

app.use(bodyParser.urlencoded({extended: false}));
app.use(bodyParser.json());

app.use(session({
  genid: () => crypto.randomBytes(16).toString('hex'),
  secret: 'hi',
  cookie: {
    secure: false,
    maxAge: 1000*60*60,
    httpOnly: false
  },
  saveUninitialized: false,
  resave: false
}));

app.use(passport.initialize());
app.use(passport.session());

app.use('/', express.static('../client/dist'));
app.use('/api', authRouter);
app.use('/api', streamRouter);
app.get('*', (_req, res) => res.sendFile('index.html', {root: '../client/dist/'}));

const server = https.createServer(credentials, app);

server.listen(port, () => {
  console.log(`App listening on port ${port}`);
});

export default app;
