import express from 'express';
import { Strategy } from 'passport-local';
import crypto from 'crypto';

const strategy = new Strategy(
  {
    usernameField: null,
    passwordField: 'password'
  },
  (_username, password, done) => {
    return done(null, password === 'hello');
});

const router = express.Router();

router.post('/auth', (req, res, next) => {
  if (req.headers.password === 'hello') {
    req.session;
    res.status(200).send();
  } else {
    res.status(401).send();
  }
  next();
});

export default router;
