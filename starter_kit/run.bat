@echo off
echo ========================================
echo   AEONMI QUANTUM STARTER KIT
echo ========================================
echo.
echo Running hello_quantum.ai...
echo.

set AEONMI_NATIVE=1
aeonmi.exe run hello_quantum.ai

echo.
echo ========================================
echo Try the other examples:
echo   aeonmi.exe run grover_search.ai
echo   aeonmi.exe run qft_pattern.ai
echo   aeonmi.exe run entanglement_demo.ai
echo ========================================
pause