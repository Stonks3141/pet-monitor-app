import express from 'express';

const router = express.Router();

router.get('/stream', (req, res) => {
  if ('session' in req) {
    res.status(200).send('stream');
  }
  else {
    res.status(401).send('unauthorized');
  }
});

export default router;
