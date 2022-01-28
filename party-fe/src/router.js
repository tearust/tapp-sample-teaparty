import Vue from 'vue'
import Router from 'vue-router'


import Home from './views/Home';
import PostMsg from './views/PostMsg';
import LogView from './views/LogView';
import LoginAccount from './views/LoginAccount';
import Test from './views/Test';

import bbs from './views/bbs';
import TappProfile from './views/TappProfile';
import MyNotification from './views/MyNotification';


Vue.use(Router);


let routers = [
  {
    path: '/',
    redirect: '/home',
  },
  {
    path: '/profile',
    name: 'profile',
    component: LoginAccount,
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
  },
  {
    path: '/test',
    name: 'test',
    component: Test,
  },
  {
    path: '/tapp_profile',
    name: 'tapp_profile',
    component: TappProfile,
  },
  {
    path: '/my_notification',
    name: 'my_notification',
    component: MyNotification,
  }
  
  
];

export default new Router({
  routes: routers
})