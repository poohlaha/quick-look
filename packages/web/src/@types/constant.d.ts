/**
 * @fileOverview 忽略文件, 防止编译报错, 使用 declare
 * @date 2023-04-12
 * @author poohlaha
 */
declare module 'react-fastclick'
declare module 'crypto-js'
// declare module 'prismjs'
declare module '*.toml'
declare module '*.png'
declare module '*.jpg'
declare module '*.jpeg'
declare module '*.mp4'
declare module '*.jpg'
declare module '*.gif'
declare const $http: any // http

declare global {
  interface Window {
    Prism: any
  }
}
