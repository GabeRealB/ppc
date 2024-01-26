import React from 'react';
import { createRoot } from 'react-dom/client';
import { StyledEngineProvider } from '@mui/material/styles';
import App from './App';

const root = createRoot(document.getElementById('root'));
root.render(
    <StyledEngineProvider injectFirst>
        <App />
    </StyledEngineProvider>
);
