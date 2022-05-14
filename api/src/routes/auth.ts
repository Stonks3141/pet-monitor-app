import express from 'express';
import passport from 'passport';
import { Strategy } from 'passport-custom';

const myPassword = '123';

declare global {
  namespace Express {
    interface User {
      id?: any;
    }
  }
}

// auth strategy, checks if password is correct
passport.use('custom', new Strategy((req, done) => {
    if (req.body.password === myPassword) {
      const user = { id: '12345' };
      done(null, user);
    }
    else {
      done(null);
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
  passport.authenticate('custom', {failureRedirect: '/urmom'}),
  (_req, res) => res.status(200).send()
);

export default router;
