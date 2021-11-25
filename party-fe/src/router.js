import Vue from 'vue'
import Router from 'vue-router'


import Home from './views/Home';
import PostMsg from './views/PostMsg';
import LogView from './views/LogView';

import bbs from './views/bbs';

Vue.use(Router);


let routers = [
  {
    path: '/',
    redirect: '/home',
  },
  {
    path: '/home',
    name: 'home',
    component: Home,
    props: {
      channel: 'default',
    }
  },
  {
    path: '/channel',
    name: 'channel',
    component: Home,
    props: {
      channel: bbs.consts.channel,
    }
  },
  {
    path: '/post_msg',
    name: 'post_msg',
    component: PostMsg,
  },
  {
    path: '/log',
    name: 'log',
    component: LogView,
  }
  
  
];

export default new Router({
  routes: routers
})