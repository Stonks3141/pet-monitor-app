import express from 'express';
import passport from 'passport';
import { Strategy } from 'passport-custom';
import argon2 from 'argon2';

const myHash = await argon2.hash('123');

declare global {
  namespace Express {
    interface User {
      id?: any;
    }
  }
}

// auth strategy, checks if password is correct
passport.use('custom', new Strategy((req, done) => {
  console.log(req.body);
  if (req.body.hash === '123') {
    const user = { id: '12345' };
    done(null, user);
  }
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

router.post('/auth',
  passport.authenticate('custom', {failureRedirect: '/uhoh'}),
  (_req, res) => res.status(200).send()
);

export default router;
