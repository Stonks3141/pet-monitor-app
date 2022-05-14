import express from 'express';

const router = express.Router();

router.get('/stream', (req, res) => {
  console.log(req.user);
  if (req.user) {
    res.status(200).send('stream');
  }
  else {
    res.status(401).send('unauthorized');
  }
});

export default router;
