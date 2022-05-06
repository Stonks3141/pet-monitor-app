import express from 'express';

const password = 'hello';

const router = express.Router();

router.post('/auth', (req, res) => {
  if (req.body === password) {
    res.send('ok');
  } else {
    res.send('no');
  }
});

export default router;
