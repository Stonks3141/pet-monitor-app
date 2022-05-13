import express from 'express';
import session from 'express-session';
import bodyParser from 'body-parser';
import passport from 'passport';
import crypto from 'crypto';
import { authRouter, streamRouter } from './routes';

const app = express();
const port = 8080;

app.use(bodyParser.urlencoded({extended: false}));
app.use(bodyParser.json());

app.use(session({
    genid: () => crypto.randomBytes(16).toString('hex'),
    secret: 'hi',
    cookie: { secure: false, maxAge: 1000*60*60 },
    saveUninitialized: false,
    resave: false
}));

app.use(passport.initialize());
app.use(passport.session());

app.use('/', express.static('../client/dist'));
app.use('/api', authRouter);
app.use('/api', streamRouter);
app.get('*', function(req, res) {
    res.sendFile('index.html', {root: '../../client/dist/'});
  });

app.listen(port, () => {
    console.log(`App listening on port ${port}`);
});

export default app;
