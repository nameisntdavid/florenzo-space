import { defineConfig } from 'sanity'
import { structureTool } from 'sanity/structure'
import { visionTool } from '@sanity/vision'
import post from './schemas/post'
import tag from './schemas/tag'

export default defineConfig({
  name: 'personal-blog',
  title: 'Personal Blog',

  projectId: '9z7f8a4p',
  dataset: process.env.SANITY_DATASET || 'production',

  plugins: [structureTool(), visionTool()],

  schema: {
    types: [post, tag],
  },
})
