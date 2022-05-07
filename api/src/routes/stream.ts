import express from 'express';

const router = express.Router();

router.get('/stream', (req, res, next) => {
  if ('session' in req) {
    res.status(200).send('stream');
  }
  else {
    res.status(401).send('unauthorized');
  }
  next();
});

export default router;
