/**
 * @fileOverview route
 * @date 2023-04-13
 * @author poohlaha
 */
import React from 'react'
import { RouteInterface } from '@router/router.interface'
import RouterUrls from '@route/router.url.toml'
const { lazy } = React

export const routes: RouteInterface[] = [
  {
    path: '*',
    exact: true,
    component: lazy(() => import(/* webpackChunkName:'lazy' */ '@pages/home/index')),
    auth: false,
    title: '首页',
  },
]
