#!/bin/sh

OS_NAME=$(uname -s)
PORT=${1:-5432}

if [ "${OS_NAME:0:5}" = "MINGW" ] || [ "${OS_NAME:0:4}" = "MSYS" ] || [ "${OS_NAME:0:6}" = "CYGWIN" ]; then
   netstat -ano | findstr :${PORT}

   if [ $? -ne 0 ]; then
       echo "No process found on port $PORT."
       exit 0
   fi

   echo "Enter the PID to release:"
   read -r pid
   powershell -Command "taskkill /F /PID ${pid}"
else
   lsof -t -i :${PORT} | xargs kill -9
fi
