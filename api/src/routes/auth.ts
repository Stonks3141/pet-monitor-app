import express from 'express';
import passport from 'passport';
import { Strategy } from 'passport-local';
import crypto from 'crypto';

// auth strategy, checks if password is correct
passport.use(new Strategy(
  {
    usernameField: null,
    passwordField: 'password'
  },
  (_username, password, done) => {
    return done(null, password === 'hello');
}));

// attaches user to request
passport.deserializeUser((id, done) => (
  done(null, { id: id })
));

// runs on login, saves user id to session
passport.serializeUser((user, done) => (
  done(null, { id: user.id })
));

const router = express.Router();
router.post('/auth', passport.authenticate('local'));

export default router;
